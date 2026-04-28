#!/usr/bin/env Rscript
# r_rng_tests.R <binary file> <rng label>
#
# Reads little-endian uint32 words from <binary file>, normalises to U[0,1),
# runs every applicable randomness test from the standard R packages, computes
# raw moments 1..10, and writes a markdown block to stdout.

suppressPackageStartupMessages({
  library(randtests)
  library(randtoolbox)
  library(tseries)
  library(moments)
  library(stats)
  # NOTE: nortest is intentionally not loaded — its tests target Normality,
  # not Uniformity, so they would always REJECT for a clean U(0,1) stream.
})

argv  <- commandArgs(trailingOnly = TRUE)
if (length(argv) != 2L)
  stop("usage: Rscript r_rng_tests.R <binary file> <rng label>", call. = FALSE)
path  <- argv[[1]]
label <- argv[[2]]
if (!file.exists(path))
  stop(sprintf("input file not found: %s", path), call. = FALSE)
if (file.info(path)$size == 0L)
  stop(sprintf("input file is empty: %s", path), call. = FALSE)
if (file.info(path)$size %% 4L != 0L)
  stop(sprintf("input file size %d is not a multiple of 4 bytes",
               file.info(path)$size), call. = FALSE)

# ---- read binary stream ------------------------------------------------------
# IMPORTANT: R's `integer` type uses INT_MIN (= -2^31) as NA_integer_, so
# `readBin(..., what=integer(), signed=TRUE)` silently turns the u32 word
# 0x80000000 into NA.  For CSPRNGs over 5e6 words this NA-poisoning hits
# ~0.12 % of runs and corrupts every downstream statistic.  We instead
# read raw bytes and reassemble each u32 from its 4 little-endian bytes
# in numeric (double) precision, where 0..2^32-1 is exact.
fi <- file(path, "rb")
sz <- file.info(path)$size
n  <- as.integer(sz / 4L)
raw_bytes <- readBin(fi, what = "raw", n = sz)
close(fi)

# Reshape into 4 x n matrix; each column is one little-endian u32.
b   <- matrix(as.numeric(as.integer(raw_bytes)), nrow = 4L)
u32_num <- b[1L, ] + 256 * (b[2L, ] + 256 * (b[3L, ] + 256 * b[4L, ]))
u   <- u32_num / 2^32                         # [0,1)
# Guard against u==0 for tests that need ]0,1[
u_nz <- pmax(u, 1 / 2^33)

# ---- helpers -----------------------------------------------------------------
fmt <- function(v) {
  if (is.null(v) || length(v) == 0) return("NA")
  if (length(v) > 1) v <- v[1]
  if (is.na(v)) return("NA")
  if (is.numeric(v)) {
    if (abs(v) < 1e-300 && v != 0) "<1e-300"
    else if (v != 0 && abs(v) < 1e-4) formatC(v, format = "e", digits = 3)
    else formatC(v, format = "f", digits = 6)
  } else as.character(v)
}

verdict <- function(p, alpha = 0.001) {
  if (is.null(p) || length(p) == 0) return("n/a")
  if (length(p) > 1) p <- p[1]
  if (is.na(p)) return("n/a")
  if (p < alpha) "REJECT" else "pass"
}

safe <- function(expr) {
  out <- tryCatch(suppressWarnings(expr),
    error = function(e) list(p.value = NA, statistic = NA,
                             error = conditionMessage(e)))
  if (is.null(out)) out <- list(p.value = NA, statistic = NA)
  out
}

cat_row <- function(test, stat, p) {
  cat(sprintf("| %s | %s | %s | %s |\n",
              test,
              if (is.null(stat)) "" else fmt(stat),
              fmt(p),
              verdict(p)))
}

# ---- header ------------------------------------------------------------------
cat(sprintf("\n## %s\n\n", label))
cat(sprintf("Sample size: %s u32 words (%.2f MB)\n\n",
            format(n, big.mark = ","), sz / 1024 / 1024))
cat(sprintf("Mean = %.6f  Var = %.6f  Min = %.6f  Max = %.6f\n\n",
            mean(u), var(u), min(u), max(u)))

cat("### Tests (alpha = 0.001 reject threshold)\n\n")
cat("| Test | Statistic | p-value | Verdict |\n")
cat("|------|-----------|---------|---------|\n")

x <- u

# NOTE: tseries::runs.test masks randtests::runs.test (only takes factors).
# All package-qualified calls below are intentional.

# ---- randtests ---------------------------------------------------------------
r <- safe(randtests::runs.test(x))
cat_row("randtests::runs.test (median)", r$statistic, r$p.value)

r <- safe(randtests::bartels.rank.test(x))
cat_row("randtests::bartels.rank.test", r$statistic, r$p.value)

r <- safe(randtests::cox.stuart.test(x))
cat_row("randtests::cox.stuart.test (trend)", r$statistic, r$p.value)

r <- safe(randtests::difference.sign.test(x))
cat_row("randtests::difference.sign.test", r$statistic, r$p.value)

r <- safe(randtests::turning.point.test(x))
cat_row("randtests::turning.point.test", r$statistic, r$p.value)

# Mann-Kendall rank.test is O(n^2): subsample to keep a per-RNG run < 1s.
sub_n <- min(length(x), 5000L)
r <- safe(randtests::rank.test(x[seq_len(sub_n)]))
cat_row(sprintf("randtests::rank.test (Mann-Kendall, n=%d)", sub_n),
        r$statistic, r$p.value)

# ---- randtoolbox: sample-based tests -----------------------------------------
r <- safe(randtoolbox::freq.test(u_nz, echo = FALSE))
cat_row("randtoolbox::freq.test (16 bins)", r$statistic, r$p.value)

r <- safe(randtoolbox::gap.test(u_nz, lower = 0, upper = 0.5, echo = FALSE))
cat_row("randtoolbox::gap.test [0,0.5)", r$statistic, r$p.value)

r <- safe(randtoolbox::serial.test(u_nz, d = 8, echo = FALSE))
cat_row("randtoolbox::serial.test (d=8)", r$statistic, r$p.value)

r <- safe(randtoolbox::poker.test(u_nz, nbcard = 5, echo = FALSE))
cat_row("randtoolbox::poker.test (5-hand)", r$statistic, r$p.value)

r <- safe(randtoolbox::order.test(u_nz, d = 4, echo = FALSE))
cat_row("randtoolbox::order.test (d=4)", r$statistic, r$p.value)

# ---- stats / tseries ---------------------------------------------------------
suppressWarnings({
  r <- safe(ks.test(u, "punif", 0, 1))
})
cat_row("stats::ks.test vs U(0,1)", r$statistic, r$p.value)

bins <- 256L
counts <- tabulate(pmin(floor(u * bins) + 1L, bins), nbins = bins)
r <- safe(chisq.test(counts))
cat_row("stats::chisq.test (256 bins)", r$statistic, r$p.value)

r <- safe(Box.test(u - 0.5, lag = 25, type = "Ljung-Box"))
cat_row("stats::Box.test (Ljung-Box, lag 25)", r$statistic, r$p.value)

fct <- factor(as.integer(u >= median(u)))
r <- safe(tseries::runs.test(fct))
cat_row("tseries::runs.test (binary)", r$statistic, r$p.value)

r <- safe(tseries::jarque.bera.test(u))
cat_row("tseries::jarque.bera.test (vs Normal*)",
        r$statistic, r$p.value)

# ---- moments -----------------------------------------------------------------
cat("\n*Note*: Jarque-Bera tests Normality; uniform output is expected to REJECT.\n")
cat("\n### Raw moments E[U^k] vs theoretical 1/(k+1)\n\n")
cat("| k | observed | theoretical | abs error |\n")
cat("|---|----------|-------------|-----------|\n")
mom <- all.moments(u, order.max = 10, central = FALSE)
for (k in 1:10) {
  obs <- mom[k + 1]
  th  <- 1 / (k + 1)
  cat(sprintf("| %d | %.8f | %.8f | %.2e |\n",
              k, obs, th, abs(obs - th)))
}

# ---- Fourier (DFT spectral) analysis -----------------------------------------
# For an i.i.d. U(0,1) sequence the centred series y_t = u_t - 1/2 is white
# noise with variance sigma^2 = 1/12.  Under the null:
#   * The periodogram I(f_k) = |Y(f_k)|^2 / N is asymptotically
#     iid Exp(sigma^2) at the Fourier frequencies f_k = k/N,
#     k = 1..floor(N/2)-1 (Bartlett 1955, Brockwell & Davis 1991).
#   * The (rescaled) periodogram ordinates 2 I(f_k) / sigma^2 are iid chi^2_2.
#   * Equivalently, P_k = I(f_k) / sigma^2 ~ Exp(1).
#
# The two summary statistics below are:
#   max_periodogram_p — the smallest p-value across all frequency bins after
#     a Bonferroni correction for the m = N/2 - 1 tested ordinates (Fisher's
#     g-style spike test; spikes from periodicity surface here).
#   spec_flatness     — the geometric/arithmetic mean ratio of the
#     periodogram (Wiener entropy).  An iid uniform stream has E[log P_k] =
#     -gamma (Euler-Mascheroni); the population flatness is exp(-gamma) ≈
#     0.561459.  Tonal / periodic signals push flatness toward 0; pure white
#     noise sits near 0.561.

cat("\n### Fourier / spectral analysis (centred series y_t = u_t - 1/2)\n\n")
y <- u - 0.5
N <- length(y)
sigma2 <- 1 / 12
ft  <- stats::fft(y)
m   <- floor(N / 2) - 1L
# discard k=0 (mean) and the Nyquist bin (k=N/2) when N is even.
P   <- (Mod(ft[2:(m + 1L)])^2) / N            # raw periodogram
Pn  <- P / sigma2                              # ~ Exp(1) under H0
maxP <- max(Pn)
# p-value of max(Pn) under H0: Pn_max ~ -log(1 - U^{1/m}); equivalently
# P[max > x] = 1 - (1 - exp(-x))^m, so p = 1 - (1 - exp(-maxP))^m.
log1p_neg_emaxP <- log1p(-exp(-maxP))
log_p_no_spike  <- m * log1p_neg_emaxP
p_spike <- -expm1(log_p_no_spike)             # 1 - exp(m * log(1-exp(-maxP)))
# Wiener spectral flatness = geomean(P) / arithmetic_mean(P)
log_geo <- mean(log(P))
arith   <- mean(P)
flatness <- exp(log_geo) / arith
# Periodogram chi^2_2 goodness-of-fit: bin Pn into 10 deciles of Exp(1).
breaks <- qexp(seq(0, 1, length.out = 11L), rate = 1)
breaks[1L]            <- -Inf
breaks[length(breaks)] <-  Inf
bin    <- findInterval(Pn, breaks, rightmost.closed = TRUE)
counts <- tabulate(bin, nbins = 10L)
exp_each <- length(Pn) / 10
chi2 <- sum((counts - exp_each)^2 / exp_each)
p_chi <- pchisq(chi2, df = 9, lower.tail = FALSE)
# Cumulative spectral KS test against Exp(1).
ks <- safe(stats::ks.test(Pn, "pexp", rate = 1))

cat("| Metric | Value |\n|--------|-------|\n")
cat(sprintf("| Periodogram bins tested (m = N/2 - 1) | %s |\n",
            format(m, big.mark = ",")))
cat(sprintf("| max normalized periodogram (P_max) | %.6f |\n", maxP))
cat(sprintf("| Bonferroni p (no spike) | %s |\n", fmt(p_spike)))
cat(sprintf("| Spectral flatness (Wiener entropy) | %.6f |\n", flatness))
cat(sprintf("| Theoretical flatness for white noise | %.6f |\n", exp(-0.5772156649)))
cat(sprintf("| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=%.3f, p=%s |\n",
            chi2, fmt(p_chi)))
cat(sprintf("| Periodogram KS vs Exp(1) | D=%.6f, p=%s |\n",
            as.numeric(ks$statistic), fmt(ks$p.value)))

cat("\n")

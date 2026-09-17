#!/usr/bin/env Rscript
# r_rng_tests.R <binary file> <rng label>
# r_rng_tests.R --self-test
#
# Reads little-endian uint32 words from <binary file>, normalises to U[0,1),
# runs every applicable randomness test from the standard R packages, computes
# raw moments 1..10, and writes a markdown block to stdout.
#
# Every scored row gets one verdict: pass (p >= 0.001), fail (p < 0.001) or
# invalid, with its reason, when the test produced no single finite p-value
# in [0, 1] or raised an error.  Every expected result must appear exactly
# once; a missing, duplicated or unnamed one is invalid.  Exit status: 0 when
# every result passes, 1 when some fail and none is invalid, 2 when any is
# invalid.  The Jarque-Bera row tests Normality, is expected to fail on
# uniform input, and does not count toward the exit status.


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

ALPHA <- 0.001

# The results every analysis reports, each exactly once.
EXPECTED <- c(
  "randtests::runs.test", "randtests::bartels.rank.test",
  "randtests::cox.stuart.test", "randtests::difference.sign.test",
  "randtests::turning.point.test", "randtests::rank.test",
  "randtoolbox::freq.test", "randtoolbox::gap.test",
  "randtoolbox::serial.test", "randtoolbox::poker.test",
  "randtoolbox::order.test", "stats::ks.test", "stats::chisq.test",
  "stats::Box.test", "tseries::runs.test", "tseries::jarque.bera.test",
  "spectral::max_spike", "spectral::heights_chisq", "spectral::heights_ks",
  "spectral::bartlett")
# Expected to fail on uniform input; excluded from the exit status.
EXPECTED_FAILURE <- "tseries::jarque.bera.test"

# pass, fail or invalid for one result, with the reason for invalid.  `reason`
# is an error message from the producer, which makes the result invalid
# whatever p is.
classify <- function(p, reason = NULL, alpha = ALPHA) {
  why <- if (!is.null(reason) && length(reason) >= 1 && nzchar(reason[[1]])) {
    reason[[1]]
  } else if (is.null(p) || length(p) == 0) {
    "no p-value"
  } else if (length(p) != 1) {
    sprintf("%d p-values", length(p))
  } else if (!is.numeric(p)) {
    "p-value is not a number"
  } else if (is.na(p)) {
    "p-value is NA or NaN"
  } else if (!is.finite(p)) {
    "p-value is infinite"
  } else if (p < 0 || p > 1) {
    sprintf("p-value %s is outside [0, 1]", format(p))
  } else {
    NULL
  }
  if (!is.null(why)) return(list(verdict = "invalid", reason = why))
  list(verdict = if (p < alpha) "fail" else "pass", reason = "")
}

verdict_text <- function(v) {
  if (v$verdict == "invalid") sprintf("invalid: %s", v$reason) else v$verdict
}

# Every result recorded so far: its key and verdict.
recorded <- list()

# Record result `key`; a blank or repeated key is itself invalid.
record <- function(key, v) {
  if (is.null(key) || !nzchar(key)) {
    v <- list(verdict = "invalid", reason = "unnamed result")
    key <- ""
  } else if (key %in% vapply(recorded, `[[`, "", "key")) {
    v <- list(verdict = "invalid", reason = sprintf("duplicate result %s", key))
  }
  recorded[[length(recorded) + 1L]] <<- list(key = key, verdict = v$verdict,
                                             reason = v$reason)
  v
}

# The recorded outcome against EXPECTED: counts, the missing and unexpected
# keys, and the exit status.
summarise <- function(recorded, expected = EXPECTED,
                      expected_failure = EXPECTED_FAILURE) {
  keys <- vapply(recorded, `[[`, "", "key")
  verdicts <- vapply(recorded, `[[`, "", "verdict")
  missing <- setdiff(expected, keys)
  unexpected <- setdiff(keys[nzchar(keys)], expected)
  counted <- !(keys %in% expected_failure)
  n_invalid <- sum(verdicts == "invalid") + length(missing) + length(unexpected)
  n_fail <- sum(verdicts[counted] == "fail")
  list(pass = sum(verdicts == "pass"), fail = n_fail, invalid = n_invalid,
       missing = missing, unexpected = unexpected,
       status = if (n_invalid > 0) 2L else if (n_fail > 0) 1L else 0L)
}

# Run `expr`; an error becomes a result whose `error` holds the message.
safe <- function(expr) {
  out <- tryCatch(suppressWarnings(expr),
    error = function(e) list(p.value = NA, statistic = NA,
                             error = conditionMessage(e)))
  if (is.null(out)) out <- list(p.value = NA, statistic = NA,
                                error = "the test returned nothing")
  out
}

# One scored table row.  The key is the label up to its first space.
cat_row <- function(test, stat, p, reason = NULL) {
  key <- sub(" .*$", "", test)
  v <- record(key, classify(p, reason))
  cat(sprintf("| %s | %s | %s | %s |\n",
              test,
              if (is.null(stat)) "" else fmt(stat),
              fmt(p),
              verdict_text(v)))
}

# ---- self-test ---------------------------------------------------------------
self_test <- function() {
  check <- function(ok, what) if (!isTRUE(ok)) stop("self-test failed: ", what, call. = FALSE)
  for (bad in list(NULL, numeric(0), c(0.2, 0.3), "0.5", NA, NaN, Inf, -Inf, -0.5, 1.25)) {
    v <- classify(bad)
    check(v$verdict == "invalid" && nzchar(v$reason),
          paste("invalid p-value", deparse(bad)))
  }
  check(classify(0)$verdict == "fail", "exact 0")
  check(classify(1)$verdict == "pass", "exact 1")
  check(classify(0.5, reason = "boom")$verdict == "invalid", "producer error with a p-value")
  r <- safe(stop("boom"))
  check(classify(r$p.value, r$error)$reason == "boom", "producer error keeps its message")
  check(classify(safe(NULL)$p.value, safe(NULL)$error)$verdict == "invalid", "producer returns nothing")

  all_pass <- lapply(EXPECTED, function(k) list(key = k, verdict = "pass", reason = ""))
  check(summarise(all_pass)$status == 0L, "all pass")
  check(summarise(list())$status == 2L &&
        length(summarise(list())$missing) == length(EXPECTED), "no results")
  check(summarise(all_pass[-1])$status == 2L, "one result missing")
  jb <- all_pass
  jb[[which(EXPECTED == EXPECTED_FAILURE)]]$verdict <- "fail"
  check(summarise(jb)$status == 0L, "the expected Jarque-Bera failure")
  one_fail <- all_pass
  one_fail[[1]]$verdict <- "fail"
  check(summarise(one_fail)$status == 1L, "one failure")
  recorded <<- list()
  record("stats::ks.test", classify(0.5))
  dup <- record("stats::ks.test", classify(0.5))
  check(dup$verdict == "invalid", "duplicate result")
  unnamed <- record("", classify(0.5))
  check(unnamed$verdict == "invalid", "unnamed result")
  check(summarise(c(all_pass, list(list(key = "extra::test", verdict = "pass", reason = ""))))$status == 2L,
        "unexpected result")
  cat("self-test passed\n")
}

argv  <- commandArgs(trailingOnly = TRUE)
if (length(argv) == 1L && argv[[1]] == "--self-test") {
  self_test()
  quit(status = 0L)
}
if (length(argv) != 2L)
  stop("usage: Rscript r_rng_tests.R <binary file> <rng label> | --self-test", call. = FALSE)

suppressPackageStartupMessages({
  library(randtests)
  library(randtoolbox)
  library(tseries)
  library(moments)
  library(stats)
  # NOTE: nortest is intentionally not loaded — its tests target Normality,
  # not Uniformity, so they would always REJECT for a clean U(0,1) stream.
})

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
# readBin counts bytes as an integer, so a stream of 2^31 bytes or more
# (536 870 912 words) cannot be read in one call.
if (is.na(sz) || sz %% 4 != 0 || sz > .Machine$integer.max)
  stop(sprintf("%s: need a whole number of words and fewer than 2^31 bytes", path))
n  <- as.integer(sz / 4L)
raw_bytes <- readBin(fi, what = "raw", n = sz)
close(fi)

# Reshape into 4 x n matrix; each column is one little-endian u32.
b   <- matrix(as.numeric(as.integer(raw_bytes)), nrow = 4L)
u32_num <- b[1L, ] + 256 * (b[2L, ] + 256 * (b[3L, ] + 256 * b[4L, ]))
u   <- u32_num / 2^32                         # [0,1)
# Guard against u==0 for tests that need ]0,1[
u_nz <- pmax(u, 1 / 2^33)

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
cat_row("randtests::runs.test (median)", r$statistic, r$p.value, r$error)

r <- safe(randtests::bartels.rank.test(x))
cat_row("randtests::bartels.rank.test", r$statistic, r$p.value, r$error)

r <- safe(randtests::cox.stuart.test(x))
cat_row("randtests::cox.stuart.test (trend)", r$statistic, r$p.value, r$error)

r <- safe(randtests::difference.sign.test(x))
cat_row("randtests::difference.sign.test", r$statistic, r$p.value, r$error)

r <- safe(randtests::turning.point.test(x))
cat_row("randtests::turning.point.test", r$statistic, r$p.value, r$error)

# Mann-Kendall rank.test is O(n^2): subsample to keep a per-RNG run < 1s.
sub_n <- min(length(x), 5000L)
r <- safe(randtests::rank.test(x[seq_len(sub_n)]))
cat_row(sprintf("randtests::rank.test (Mann-Kendall, n=%d)", sub_n),
        r$statistic, r$p.value, r$error)

# ---- randtoolbox: sample-based tests -----------------------------------------
r <- safe(randtoolbox::freq.test(u_nz, echo = FALSE))
cat_row("randtoolbox::freq.test (16 bins)", r$statistic, r$p.value, r$error)

r <- safe(randtoolbox::gap.test(u_nz, lower = 0, upper = 0.5, echo = FALSE))
# randtoolbox::gap.test extends bins out to expected count ~0.1, well below
# the Cochran-rule threshold of expected >= 5 (Cochran 1954; Knuth TAOCP §3.3.1).
# A single observation in a sparse tail bin contributes (1 - lambda)^2 / lambda
# to chi^2 — a per-RNG false-positive rate of ~10^-3 that does not shrink with
# sample size (the number of unsafe bins stays ~5 for any n).  Re-aggregate
# tail bins until each surviving bin has expected >= 5, then recompute chi^2
# and the p-value on that merged distribution.
if (!is.null(r$observed) && !is.null(r$expected) && length(r$observed) > 0
    && all(is.finite(r$expected)) && all(is.finite(r$observed))) {
  obs  <- as.numeric(r$observed)
  exp_ <- as.numeric(r$expected)
  # The geometric expected sequence is monotonically decreasing, so the set
  # of "safe" bins (expected >= 5) is a contiguous prefix.  Compute the
  # last safe index, then merge the entire unsafe tail into one O(N) sum
  # — avoids the O(N^2) pop-from-end pattern that hung on CounterRng,
  # whose gap.test return has length(observed) = 5,000,000.
  keep <- which(exp_ >= 5)
  if (length(keep) == 0L) {
    # The whole distribution is sparse — chi^2 is degenerate, fall through.
    cat_row("randtoolbox::gap.test [0,0.5)", r$statistic, r$p.value, r$error)
  } else {
    last_keep <- max(keep)
    if (last_keep < length(exp_)) {
      tail_idx <- (last_keep + 1L):length(exp_)
      exp_[last_keep] <- exp_[last_keep] + sum(exp_[tail_idx])
      obs[last_keep]  <- obs[last_keep]  + sum(obs[tail_idx])
      exp_ <- exp_[seq_len(last_keep)]
      obs  <- obs[seq_len(last_keep)]
    }
    if (length(exp_) >= 2L) {
      chi2 <- sum((obs - exp_)^2 / exp_)
      df   <- length(exp_) - 1L
      pv   <- pchisq(chi2, df = df, lower.tail = FALSE)
      cat_row(sprintf("randtoolbox::gap.test [0,0.5) (Cochran-trimmed, df=%d)", df),
              chi2, pv)
    } else {
      cat_row("randtoolbox::gap.test [0,0.5) (Cochran-trimmed)",
              r$statistic, r$p.value, r$error)
    }
  }
} else {
  cat_row("randtoolbox::gap.test [0,0.5)", r$statistic, r$p.value, r$error)
}

r <- safe(randtoolbox::serial.test(u_nz, d = 8, echo = FALSE))
cat_row("randtoolbox::serial.test (d=8)", r$statistic, r$p.value, r$error)

r <- safe(randtoolbox::poker.test(u_nz, nbcard = 5, echo = FALSE))
cat_row("randtoolbox::poker.test (5-hand)", r$statistic, r$p.value, r$error)

r <- safe(randtoolbox::order.test(u_nz, d = 4, echo = FALSE))
cat_row("randtoolbox::order.test (d=4)", r$statistic, r$p.value, r$error)

# ---- stats / tseries ---------------------------------------------------------
suppressWarnings({
  r <- safe(ks.test(u, "punif", 0, 1))
})
cat_row("stats::ks.test vs U(0,1)", r$statistic, r$p.value, r$error)

bins <- 256L
counts <- tabulate(pmin(floor(u * bins) + 1L, bins), nbins = bins)
r <- safe(chisq.test(counts))
cat_row("stats::chisq.test (256 bins)", r$statistic, r$p.value, r$error)

r <- safe(Box.test(u - 0.5, lag = 25, type = "Ljung-Box"))
cat_row("stats::Box.test (Ljung-Box, lag 25)", r$statistic, r$p.value, r$error)

fct <- factor(as.integer(u >= median(u)))
r <- safe(tseries::runs.test(fct))
cat_row("tseries::runs.test (binary)", r$statistic, r$p.value, r$error)

r <- safe(tseries::jarque.bera.test(u))
cat_row("tseries::jarque.bera.test (vs Normal*)",
        r$statistic, r$p.value, r$error)

# ---- moments -----------------------------------------------------------------
cat("\n*Note*: Jarque-Bera tests Normality; uniform output is expected to fail it.\n")
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
#   max-spike exact p — the exact p-value of the largest normalised ordinate
#     under H0 via the max order statistic of m = N/2 - 1 iid Exp(1) draws,
#     P[max > x] = 1 - (1 - e^{-x})^m (Fisher's g-style spike test; spikes
#     from periodicity surface here).
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
# Wiener spectral flatness = geomean(P) / arithmetic_mean(P).
# A constant stream has zero energy at every nonzero frequency: the
# periodogram is identically 0, log(P) is -Inf everywhere, and 0/0 would
# yield NaN — emit NA instead (with a note in the output row).
log_geo <- mean(log(P))
arith   <- mean(P)
flatness <- if (arith == 0) NA_real_ else exp(log_geo) / arith
# Periodogram chi^2_2 goodness-of-fit: bin Pn into 10 deciles of Exp(1).
breaks <- qexp(seq(0, 1, length.out = 11L), rate = 1)
breaks[1L]            <- -Inf
breaks[length(breaks)] <-  Inf
bin    <- findInterval(Pn, breaks, rightmost.closed = TRUE)
counts <- tabulate(bin, nbins = 10L)
exp_each <- length(Pn) / 10
chi2 <- sum((counts - exp_each)^2 / exp_each)
p_chi <- pchisq(chi2, df = 9, lower.tail = FALSE)
# Distribution of the periodogram heights: KS of the unordered Pn against
# Exp(1).  Sorting discards frequency order, so this tests the marginal law
# of the heights, not where along the spectrum the power lies.  It is also
# conservative for uniform data: by Parseval the heights sum to a multiple of
# sum(y^2), whose variance for uniform y is 2/5 of the Gaussian value, so the
# heights are more evenly spread than iid Exp(1).  On R's Mersenne Twister it
# rejected 0.15% of 2 000 streams of 10^6 words at 0.01, and none of 1 000
# streams of 5*10^6 words.
ks <- safe(stats::ks.test(Pn, "pexp", rate = 1))
# Bartlett's cumulative periodogram: with iid Exp(1) heights, the partial
# sums C_j = (Pn_1 + ... + Pn_j) / sum(Pn), j = 1 ... m - 1, in frequency
# order, are distributed as the order statistics of m - 1 independent
# Uniform(0, 1) values, so a KS test of them against Uniform(0, 1) detects
# power concentrated in any band.  M. S. Bartlett, "An Introduction to
# Stochastic Processes", Cambridge University Press, 1955.
cum <- cumsum(Pn)
# The same null runs rejected at 0.01 in 1.30% (10^6 words, 2 000 streams)
# and 0.90% (5*10^6 words, 1 000 streams); KS of those p-values against
# uniformity gave 0.16 and 0.19.
bartlett <- if (cum[m] > 0) safe(stats::ks.test(cum[-m] / cum[m], "punif")) else NULL

cat("| Metric | Value |\n|--------|-------|\n")
cat(sprintf("| Periodogram bins tested (m = N/2 - 1) | %s |\n",
            format(m, big.mark = ",")))
cat(sprintf("| max normalized periodogram (P_max) | %.6f |\n", maxP))
# The exact max-order-statistic p 1-(1-e^{-x})^m, not a Bonferroni bound.
cat(sprintf("| Max-spike exact p (no spike) | %s (%s) |\n", fmt(p_spike),
            verdict_text(record("spectral::max_spike", classify(p_spike)))))
if (is.na(flatness)) {
  cat("| Spectral flatness (Wiener entropy) | NA (all-zero periodogram; degenerate/constant stream) |\n")
} else {
  cat(sprintf("| Spectral flatness (Wiener entropy) | %.6f |\n", flatness))
}
cat(sprintf("| Theoretical flatness for white noise | %.6f |\n", exp(-0.5772156649)))
cat(sprintf("| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=%.3f, p=%s (%s) |\n",
            chi2, fmt(p_chi),
            verdict_text(record("spectral::heights_chisq", classify(p_chi)))))
cat(sprintf("| Periodogram height KS vs Exp(1) | D=%.6f, p=%s (%s) |\n",
            as.numeric(ks$statistic), fmt(ks$p.value),
            verdict_text(record("spectral::heights_ks", classify(ks$p.value, ks$error)))))
if (is.null(bartlett)) {
  v <- record("spectral::bartlett",
              list(verdict = "invalid", reason = "all-zero periodogram"))
  cat(sprintf("| Cumulative periodogram KS (Bartlett) | NA (%s) |\n", verdict_text(v)))
} else {
  cat(sprintf("| Cumulative periodogram KS (Bartlett) | D=%.6f, p=%s (%s) |\n",
              as.numeric(bartlett$statistic), fmt(bartlett$p.value),
              verdict_text(record("spectral::bartlett",
                                  classify(bartlett$p.value, bartlett$error)))))
}

# ---- outcome -----------------------------------------------------------------
outcome <- summarise(recorded)
cat(sprintf("\n**Outcome**: %d pass, %d fail, %d invalid",
            outcome$pass, outcome$fail, outcome$invalid))
if (length(outcome$missing) > 0)
  cat(sprintf("; not measured: %s", paste(outcome$missing, collapse = ", ")))
if (length(outcome$unexpected) > 0)
  cat(sprintf("; unexpected: %s", paste(outcome$unexpected, collapse = ", ")))
cat(" (Jarque-Bera excluded from the fail count)\n\n")
quit(status = outcome$status)

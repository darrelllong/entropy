#!/usr/bin/env Rscript
# r_gap_test_diagnostic.R
#
# Reproduces the bin-by-bin diagnosis of `randtoolbox::gap.test` that
# revealed the Cochran-rule violation discussed in R-REPORT.md.
#
# Usage:
#   Rscript scripts/r_gap_test_diagnostic.R [<binary stream>]
#
# With no argument the script runs gap.test on five independent
# `runif(5e6)` samples, demonstrating that the test passes cleanly on
# bona-fide i.i.d. uniforms.  With a path it reads a little-endian u32
# stream (as produced by `target/release/dump_rng`), normalises to U(0,1),
# runs gap.test, and prints
#
#   * the raw chi^2 / p-value reported by randtoolbox,
#   * the top-5 bins by chi^2 contribution,
#   * the Cochran-trimmed chi^2 / p-value (drop tail bins with expected < 5),
#   * the same merge-up-from-the-tail variant used by `r_rng_tests.R`.
#
# This is the script that produced the SpongeBob / Squidward false-positive
# diagnosis: total chi^2 was dominated by a single observation in a bin
# whose expected count was ~10^-3.

suppressPackageStartupMessages(library(randtoolbox))

argv <- commandArgs(trailingOnly = TRUE)

baseline <- function(N = 5e6, trials = 5L) {
  cat(sprintf("baseline: %d trials of runif(%g) on R's own RNG\n", trials, N))
  for (i in seq_len(trials)) {
    u <- runif(N)
    r <- gap.test(u, lower = 0, upper = 0.5, echo = FALSE)
    cat(sprintf("  trial %d: chi2=%8.3f  p=%.3e  bins=%d\n",
                i, r$statistic, r$p.value, length(r$observed)))
  }
}

read_u32_stream <- function(path) {
  fi <- file(path, "rb")
  sz <- file.info(path)$size
  b  <- readBin(fi, what = "raw", n = sz)
  close(fi)
  m   <- matrix(as.numeric(as.integer(b)), nrow = 4L)
  u32 <- m[1L, ] + 256 * (m[2L, ] + 256 * (m[3L, ] + 256 * m[4L, ]))
  u32 / 2^32
}

cochran_trim <- function(observed, expected, threshold = 5) {
  i <- length(expected)
  while (i > 1L && expected[i] < threshold) {
    expected[i - 1L] <- expected[i - 1L] + expected[i]
    observed[i - 1L] <- observed[i - 1L] + observed[i]
    expected <- expected[-i]
    observed <- observed[-i]
    i <- i - 1L
  }
  list(observed = observed, expected = expected)
}

diagnose_stream <- function(path) {
  cat(sprintf("\nstream: %s\n", path))
  u <- read_u32_stream(path)
  cat(sprintf("  N = %s   mean = %.6f   sum(u >= 0.5) / N = %.6f\n",
              format(length(u), big.mark = ","),
              mean(u),
              mean(u >= 0.5)))
  r <- gap.test(u, lower = 0, upper = 0.5, echo = FALSE)
  cat(sprintf("  raw randtoolbox: chi2=%.3f  df=%d  p=%.3e\n",
              r$statistic, length(r$observed) - 1L, r$p.value))

  contrib <- (r$observed - r$expected)^2 / pmax(r$expected, 1e-300)
  ord <- order(contrib, decreasing = TRUE)[seq_len(min(5L, length(contrib)))]
  cat("  top-5 contributing bins (gap-length, observed, expected, chi2):\n")
  for (k in ord) {
    cat(sprintf("    g=%2d  obs=%6d  exp=%9.3g  chi2=%9.3f\n",
                k - 1L, r$observed[k], r$expected[k], contrib[k]))
  }

  trim <- cochran_trim(r$observed, r$expected, threshold = 5)
  if (length(trim$expected) >= 2L) {
    chi2 <- sum((trim$observed - trim$expected)^2 / trim$expected)
    df   <- length(trim$expected) - 1L
    pv   <- pchisq(chi2, df = df, lower.tail = FALSE)
    cat(sprintf("  Cochran-trimmed:  chi2=%.3f  df=%d  p=%.6f\n",
                chi2, df, pv))
  } else {
    cat("  Cochran-trim left <2 bins — test undefined\n")
  }
}

if (length(argv) == 0L) {
  baseline()
} else {
  for (p in argv) {
    if (!file.exists(p)) {
      cat(sprintf("skipping missing file: %s\n", p))
      next
    }
    diagnose_stream(p)
  }
}

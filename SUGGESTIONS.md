# Suggestions

Improvements beyond the open defects in [AUDIT.md](AUDIT.md).

## Calibration practice

- Predeclare thresholds, uncertainty targets, sample sizes and training and
  held-out seeds before a campaign; keep the driver, parameters, table digests
  and raw summaries with its result.  At α = 0.001, 100 000 null replicates
  give about 100 rejections, still roughly ±20% at 95% confidence.
- Validate on held-out streams from several generator families and on a saved
  corpus (`run_tests --corpus`).  Report false-alarm rate, power and cost
  together; views of one stream share data and are not independent
  replicates.
- For a simulated discrete law, carry the table's uncertainty into the
  decision or bound its domain by measured false-alarm rates.
- Combine dependent results with Holm or Bonferroni bounds; never multiply,
  Fisher-combine or Šidák-correct p-values that share input.  Conservative
  anytime p-values must not enter tests that assume uniform p-values.

## Probability kernels

- Give each numerical kernel (`igamc`, `normal_quantile`, the Kolmogorov–
  Smirnov and Anderson–Darling distributions) a stated domain and error
  target, with independent reference values at the battery's actual degrees
  of freedom and rejection thresholds as well as at extreme shapes.
- Share R-report fixtures and the pass/fail/invalid contract with
  cryptography's cipher report.

## Results

- Name the calibration table or null model version in each result's
  `Statistic`, and record the exact input range a suite read in `--json`.

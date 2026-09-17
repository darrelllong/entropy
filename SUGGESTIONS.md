# Suggestions

Improvements beyond the open defects in [AUDIT.md](AUDIT.md).

## Results

- `run_tests --json` gives each result's unrounded p-value, status and the
  offset of its suite's input.  The statistic, degrees of freedom and null
  model still live only in the note; give them their own fields.

## Calibration practice

- Choose sample sizes and tolerances before running validation seeds.  At
  α = 0.001, 100 000 null replicates give about 100 expected rejections, still
  roughly ±20% at 95% confidence.
- Validate on held-out streams from several generator families, and report
  false-alarm rate, power and cost together.  `run_tests --alternatives`
  measures power against five fixed defects at the battery's one sample
  size; power curves over defect strength and sample size remain to be
  measured.
- Calibrate the battery's decision, not only each marginal test.
- Combine dependent results with Holm or Bonferroni bounds; never multiply,
  Fisher-combine or Šidák-correct p-values that share input.

## A sequential test with an error guarantee

A predictor q_t for the next bit, built only from past bits, defines
E_t = E_{t−1}·2·q_t^{x_t}(1 − q_t)^{1−x_t}, a nonnegative martingale under fair
independent bits; by Ville's inequality it exceeds 1/α with probability at
most α, however long the test runs (S. R. Howard et al., *Annals of
Statistics* 2021).  Mixtures of Markov and context-tree predictors (Willems,
Shtarkov and Tjalkens 1995) could detect bias, lag structure and short periods
with one guarantee instead of hundreds of p-values.  E-values are not uniform
p-values and must not be fed into the KS machinery.

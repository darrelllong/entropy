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

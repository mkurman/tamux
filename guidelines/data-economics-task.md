---
name: data-economics-task
description: Apply investment science to data acquisition — marginal value curves, portfolio diversification, acquisition cost-benefit, market valuation, and investment timing optimization. Data as a capital asset.
recommended_skills: [cost-model-task, data-attribution-task, embedding-analysis]
recommended_guidelines: [data-portfolio-theory-task, cost-model-task]
---

## Overview

Data is a capital asset. Every dollar spent on data should be allocated where it produces the highest marginal return. This guideline applies investment frameworks — marginal analysis, portfolio theory, and cost-benefit optimization — to data acquisition.

## Phase 1: Marginal Data Value Curves

```python
def marginal_value_curve(model, data_source, eval_task, sample_sizes=[100, 500, 2000, 10000, 50000]):
    from scipy.optimize import curve_fit
    
    scores, costs = [], []
    per_ex_cost = data_source.get("cost_per_example", 0.01)
    
    for n in sample_sizes:
        model.fit(data_source["data"].sample(n))
        scores.append(evaluate(model, eval_task))
        costs.append(n * per_ex_cost)
    
    def log_fit(x, a, b): return a * np.log(x + 1) + b
    popt, _ = curve_fit(log_fit, costs, scores)
    a, b = popt
    
    # Marginal value at current spend: derivative = a / (cost + 1)
    current_cost = sum(s["cost"] for s in data_sources if s["active"])
    marginal_value = a / (current_cost + 1)
    saturation_cost = (a / 0.001) - 1  # where marginal value < 0.1%
    
    return {"marginal_value": marginal_value, "saturation_cost": saturation_cost,
            "recommendation": "INVEST" if marginal_value > 0.001 else "DIVEST" if current_cost > saturation_cost else "MAINTAIN"}
```

## Phase 2: Portfolio Diversification

```python
def diversification_score(corr_matrix, allocations):
    n = len(allocations)
    if n <= 1: return 1.0
    total = sum(allocations.values())
    weights = {k: v/total for k, v in allocations.items()}
    anti_corr = sum(weights.get(i,0) * weights.get(j,0) * (1 - corr_matrix[i,j])
                    for i in range(n) for j in range(i+1, n))
    return anti_corr / (n * (n-1) / 200) if n > 1 else 0

def diversified_acquisition(portfolio, candidate):
    avg_corr = np.mean([candidate["correlations"].get(k, 0.5) for k in portfolio])
    benefit = 1 - avg_corr
    return {"benefit": benefit, "priority": "HIGH" if benefit > 0.7 else "MEDIUM" if benefit > 0.4 else "LOW"}
```

## Phase 3: Acquisition Cost-Benefit

| Data Source | Collection Cost | Curation Cost | Expected Model Gain | ROI | Acquire? |
|-------------|----------------|---------------|---------------------|-----|----------|
| Source A | $5K | $2K | +2.3% accuracy | 4.6x | ✅ Yes |
| Source B | $25K | $5K | +1.1% accuracy | 0.9x | ❌ No |
| Source C | $1K | $1K | +0.8% accuracy | 4.0x | ✅ Yes |

## Phase 4: Market Valuation

```python
def value_data_asset(dataset, model, eval_task, replacement_cost_per_ex=0.50):
    n = len(dataset)
    perf_with = evaluate(model_trained_with(dataset), eval_task)
    perf_without = evaluate(model_trained_without(dataset), eval_task)
    business_value_per_point = model.get("value_per_accuracy_point", 100000)
    contribution_val = (perf_with - perf_without) * business_value_per_point
    replacement_cost = n * replacement_cost_per_ex
    uniqueness_premium = 1.0 + dataset.get("uniqueness", 0.5) * 0.5
    fair_value = 0.5 * contribution_val + 0.3 * replacement_cost + 0.2 * (contribution_val * uniqueness_premium)
    return {"fair_value": fair_value, "price_per_example": fair_value / max(n, 1)}
```

## Quality Gate

- Marginal value curve computed for every source > 5% of budget.
- Diversification score > 0.3 — concentrated portfolios are fragile.
- Acquisition prioritized by ROI, not just cost.
- Data valuation performed before any licensing deal > $10K.

---
name: data-portfolio-theory-task
description: Apply portfolio theory to data acquisition — marginal value optimization, acquisition priority scoring, data diversification, and fair market valuation for data licensing. Treat data as a capital asset, not a cost center.
recommended_skills:
  - cost-model-task
  - data-attribution-task
  - embedding-analysis
recommended_guidelines:
  - cost-model-task
  - data-strategy-foundation-models-task
  - data-attribution-task
  - training-data-design-principles
---

## Overview

Data is a capital asset, not a cost center. Every dollar spent on data acquisition should be allocated where it produces the highest marginal improvement in model capability. Portfolio theory — originally developed for financial assets — provides the mathematical framework for optimizing data investment. This guideline treats data as an asset class with measurable return, risk, and diversification properties.

## Phase 1: Marginal Value per Acquisition Dollar

### The Fundamental Question

For every candidate data source, answer: "If I spend $1 more on this source, how much does my model improve?"

```python
def marginal_value_curve(model, data_source, eval_tasks, 
                          sample_sizes=[100, 500, 1000, 5000, 10000, 50000]):
    """
    Measure model performance as a function of training data volume
    from a specific source. Returns the value curve and diminishing returns point.
    """
    performances = []
    costs = []
    per_example_cost = data_source.get("cost_per_example", 0.01)
    
    for n in sample_sizes:
        # Subsample
        subset = data_source["data"].sample(n)
        
        # Train model on subset
        model_subset = train_model(model, subset)
        
        # Evaluate
        scores = {task: evaluate(model_subset, task) for task in eval_tasks}
        mean_score = np.mean(list(scores.values()))
        
        performances.append(mean_score)
        costs.append(n * per_example_cost)
    
    # Fit value curve: performance = f(cost)
    # Use logarithmic fit: perf ≈ a * log(cost) + b
    from scipy.optimize import curve_fit
    
    def log_fit(x, a, b):
        return a * np.log(x + 1) + b
    
    popt, _ = curve_fit(log_fit, costs, performances)
    a, b = popt
    
    # Marginal value: derivative of log fit = a / (cost + 1)
    current_cost = data_source.get("current_cost", 0)
    marginal_value = a / (current_cost + 1)
    
    # Diminishing returns: where marginal value drops below threshold
    threshold = 0.001  # 0.1% improvement per $1000
    saturation_cost = (a / threshold) - 1 if a > 0 else float("inf")
    
    return {
        "source": data_source["name"],
        "fit_params": {"a": a, "b": b},
        "marginal_value_at_current": marginal_value,
        "saturation_cost": saturation_cost,
        "diminishing_returns": current_cost > saturation_cost * 0.8,
        "curve": {"costs": costs, "performances": performances},
        "recommendation": (
            "INVEST" if marginal_value > threshold and not (current_cost > saturation_cost * 0.8)
            else "MAINTAIN" if marginal_value > threshold * 0.5
            else "DIVEST"
        ),
    }
```

### Value Curve Interpretation

| Curve Shape | What It Means | Action |
|-------|-------|-------|
| Steep log growth | Each dollar still improves the model significantly | Increase investment |
| Flattening log curve | Diminishing returns — marginal value dropping | Maintain current investment |
| Flat / noisy | Saturated — more data won't help | Stop acquisition, reallocate budget |
| Negative slope | More data HURTS (noise, distribution shift) | Audit data quality immediately |

## Phase 2: Portfolio Diversification

### The Data Correlation Matrix

Data sources are not independent. Two sources might be near-duplicates — buying both wastes money.

```python
def data_correlation_matrix(sources, sample_size=1000):
    """
    Measure how redundant data sources are in embedding space.
    High correlation = waste of acquisition budget.
    """
    from scipy.spatial.distance import jensenshannon
    
    n = len(sources)
    corr_matrix = np.zeros((n, n))
    
    embeddings = {}
    for i, source in enumerate(sources):
        subset = source["data"].sample(min(sample_size, len(source["data"])))
        embeddings[i] = embed(subset)
    
    for i in range(n):
        for j in range(i, n):
            if i == j:
                corr_matrix[i, j] = 1.0
            else:
                # Fréchet distance as a correlation proxy
                fd = _frechet_distance(embeddings[i], embeddings[j])
                # Normalize: 0 = identical, 1 = completely different
                emb_i_mean = embeddings[i].mean(axis=0)
                emb_j_mean = embeddings[j].mean(axis=0)
                max_dist = np.linalg.norm(emb_i_mean) + np.linalg.norm(emb_j_mean)
                normalized_fd = min(fd / max_dist, 1.0) if max_dist > 0 else 0
                corr_matrix[i, j] = 1 - normalized_fd
                corr_matrix[j, i] = 1 - normalized_fd
    
    return corr_matrix

def diversification_score(corr_matrix, current_allocation):
    """
    How diversified is the current data portfolio?
    Score = weighted average of (1 - correlation) between allocated sources.
    Higher = more diversified = better risk-adjusted return.
    """
    n = len(current_allocation)
    if n <= 1:
        return 1.0  # trivial
    
    total_alloc = sum(current_allocation.values())
    weights = {k: v / total_alloc for k, v in current_allocation.items()}
    
    weighted_anticorr = 0
    pairs = 0
    for i in range(n):
        for j in range(i + 1, n):
            anti_corr = 1 - corr_matrix[i, j]
            weight = weights.get(i, 0) * weights.get(j, 0)
            weighted_anticorr += anti_corr * weight
            pairs += 1
    
    return weighted_anticorr / (pairs * 0.01) if pairs > 0 else 0

def diversify_recommendation(corr_matrix, current_allocation, candidate_sources):
    """
    Which uncorrelated source should you acquire to improve diversification?
    """
    recommendations = []
    
    for candidate_idx, candidate in enumerate(candidate_sources):
        # Average correlation with current portfolio
        avg_corr = np.mean([
            corr_matrix[candidate_idx, i] 
            for i in current_allocation.keys()
            if i in range(len(corr_matrix))
        ])
        
        # Diversification benefit = how uncorrelated it is
        benefit = 1 - avg_corr
        
        recommendations.append({
            "source": candidate["name"],
            "correlation_with_portfolio": float(avg_corr),
            "diversification_benefit": float(benefit),
            "priority": "HIGH" if benefit > 0.7 else "MEDIUM" if benefit > 0.4 else "LOW",
        })
    
    return sorted(recommendations, key=lambda x: -x["diversification_benefit"])
```

## Phase 3: Acquisition Priority Scoring

### The Multi-Factor Priority Model

```python
def score_acquisition_priority(candidate_sources, current_portfolio, model, eval_tasks):
    """
    Score each candidate data source by:
    - Marginal value (how much will it improve the model?)
    - Diversification (how different is it from what we have?)
    - Cost efficiency (how cheap per unit of improvement?)
    - Reliability (can we actually get it? quality guaranteed?)
    - Strategic alignment (does it support long-term goals?)
    """
    scores = []
    
    for source in candidate_sources:
        # Factor 1: Marginal value (estimated from similar sources or small pilot)
        mv = estimate_marginal_value(model, source, eval_tasks, pilot_size=500)
        
        # Factor 2: Diversification benefit
        div = estimate_diversification(source, current_portfolio)
        
        # Factor 3: Cost efficiency
        cost_per_ex = source.get("cost_per_example", 0.01)
        total_cost = source.get("estimated_volume", 100000) * cost_per_ex
        cost_efficiency = mv["estimated_gain"] / (total_cost + 1) if total_cost > 0 else 0
        
        # Factor 4: Reliability
        reliability = source.get("reliability_score", 0.5)  # 0-1
        
        # Factor 5: Strategic alignment
        strategic = source.get("strategic_alignment", 0.5)  # 0-1
        
        # Weighted composite
        priority_score = (
            0.30 * mv["estimated_gain"] +
            0.25 * div +
            0.20 * cost_efficiency * 100 +  # scale for comparability
            0.15 * reliability +
            0.10 * strategic
        )
        
        scores.append({
            "source": source["name"],
            "priority_score": float(priority_score),
            "marginal_value": mv,
            "diversification_benefit": div,
            "cost_efficiency": cost_efficiency,
            "reliability": reliability,
            "strategic_alignment": strategic,
            "recommended_allocation_pct": float(priority_score / sum(
                s["priority_score"] for s in [{"priority_score": ps} for ps in [priority_score]])),
        })
    
    return sorted(scores, key=lambda x: -x["priority_score"])
```

## Phase 4: Data Asset Valuation

### Fair Market Value for Data Licensing

```python
def value_data_asset(dataset, model, eval_tasks, comparable_transactions=None):
    """
    Estimate fair market value of a dataset based on:
    - Marginal contribution to model performance
    - Replacement cost (what would it cost to collect?)
    - Comparable transactions (what have similar datasets sold for?)
    - Uniqueness premium (how hard is this to replicate?)
    """
    # 1. Contribution-based valuation
    perf_with = evaluate(model_trained_with(dataset), eval_tasks)
    perf_without = evaluate(model_trained_without(dataset), eval_tasks)
    delta = perf_with - perf_without
    
    # Convert performance delta to business value
    business_value_per_point = model.get("business_value_per_accuracy_point", 100000)  # $
    contribution_value = delta * business_value_per_point
    
    # 2. Replacement cost
    n_examples = len(dataset)
    cost_per_example = dataset.get("annotation_cost_per_example", 0.50)
    collection_cost = n_examples * cost_per_example
    curation_cost = n_examples * 0.10  # QC overhead
    replacement_cost = collection_cost + curation_cost
    
    # 3. Comparable transactions (if available)
    comp_value = None
    if comparable_transactions:
        comp_value = np.median([t["price_per_example"] for t in comparable_transactions]) * n_examples
    
    # 4. Uniqueness premium
    uniqueness = dataset.get("uniqueness_score", 0.5)  # 0 = easily replicable, 1 = impossible
    uniqueness_premium = 1.0 + uniqueness * 0.5  # up to 50% premium
    
    # Weighted valuation
    if comp_value:
        fair_value = (
            0.40 * contribution_value +
            0.30 * replacement_cost +
            0.20 * comp_value +
            0.10 * (contribution_value * uniqueness_premium)
        )
    else:
        fair_value = (
            0.50 * contribution_value +
            0.30 * replacement_cost +
            0.20 * (contribution_value * uniqueness_premium)
        )
    
    return {
        "contribution_value": contribution_value,
        "replacement_cost": replacement_cost,
        "comparable_value": comp_value,
        "uniqueness_premium_factor": uniqueness_premium,
        "fair_market_value": fair_value,
        "price_per_example": fair_value / max(n_examples, 1),
        "valuation_range": (fair_value * 0.7, fair_value * 1.3),
        "valuation_confidence": "HIGH" if comp_value else "MEDIUM",
    }
```

## Phase 5: Portfolio Monitoring

### The Data Investment Dashboard

```python
def portfolio_dashboard(portfolio):
    """
    Monitor data portfolio health metrics.
    """
    total_invested = sum(p["cost"] for p in portfolio)
    total_value = sum(p.get("current_value", 0) for p in portfolio)
    
    return {
        "total_invested": total_invested,
        "total_value": total_value,
        "roi": (total_value - total_invested) / max(total_invested, 1),
        "n_sources": len(portfolio),
        "diversification": diversification_score(
            data_correlation_matrix(portfolio), 
            {i: p["cost"] for i, p in enumerate(portfolio)}
        ),
        "saturated_sources": [p["name"] for p in portfolio if p.get("diminishing_returns")],
        "high_roi_sources": [p["name"] for p in portfolio if p.get("roi", 0) > 2.0],
        "recommendations": [],
    }
```

## Quality Gate

- Marginal value curve computed for every significant data source (> 5% of budget).
- Correlation matrix computed between all pairs of data sources.
- Diversification score > 0.3 — concentrated portfolios are fragile.
- Saturated sources (> 80% of saturation cost) are flagged for divestment.
- Acquisition priority scores computed for all new candidates before purchase.
- Data asset valuation performed before any licensing deal > $10K.
- Portfolio rebalanced quarterly.

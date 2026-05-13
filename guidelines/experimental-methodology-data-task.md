---
name: experimental-methodology-data-task
description: Curate data for experimental design, human subject research, surveys/questionnaires, and qualitative data integration — hypothesis validation, control group construction, power analysis, IRB compliance, recruitment bias, survey fatigue, Likert scale interpretation, coding scheme validation, and thematic saturation.
recommended_skills: [bias-audit, label-quality-audit, annotation-management-task]
recommended_guidelines: [evaluation-dataset-design-task, annotation-economics-task, cross-validation-strategy-task]
---

## Experimental Design Science

### Hypothesis Validation

```python
def validate_hypothesis(hypothesis):
    """Is the hypothesis testable, falsifiable, and measurable?"""
    checks = {
        "testable": "measure" in hypothesis.lower() or "compare" in hypothesis.lower() or "difference" in hypothesis.lower(),
        "falsifiable": "no" in hypothesis.lower() or "not" in hypothesis.lower() or "null" in hypothesis.lower(),
        "measurable": any(term in hypothesis.lower() for term in ["accuracy", "error", "rate", "score", "time", "count"]),
        "specific": len(hypothesis.split()) > 10 and "maybe" not in hypothesis.lower(),
    }
    return {"checks": checks, "valid": all(checks.values()),
            "issues": [k for k, v in checks.items() if not v]}

def validate_control_groups(treatment_groups, control_groups):
    """Are control groups properly constructed?"""
    issues = []
    for ctrl_name, ctrl_data in control_groups.items():
        # Check: no treatment contamination
        for trt_name, trt_data in treatment_groups.items():
            overlap = len(set(trt_data.get("ids", [])) & set(ctrl_data.get("ids", [])))
            if overlap > 0:
                issues.append(f"CONTAMINATION: {overlap} subjects in both {trt_name} and {ctrl_name}")
        # Check: comparable demographics
        ctrl_demo = ctrl_data.get("demographics", {})
        trt_demo_avg = {k: np.mean([t.get("demographics", {}).get(k, 0) for t in treatment_groups.values()]) for k in ctrl_demo}
        for key in ctrl_demo:
            if abs(ctrl_demo[key] - trt_demo_avg.get(key, 0)) / max(abs(trt_demo_avg.get(key, 1)), 1) > 0.15:
                issues.append(f"IMBALANCE: control differs from treatment on {key}")
    return {"valid": len(issues) == 0, "issues": issues}
```

### Power Analysis

```python
def validate_statistical_power(n_samples, effect_size, alpha=0.05, target_power=0.8):
    from statsmodels.stats.power import TTestIndPower
    analysis = TTestIndPower()
    achieved_power = analysis.solve_power(effect_size=effect_size, nobs1=n_samples, alpha=alpha)
    min_n = analysis.solve_power(effect_size=effect_size, power=target_power, alpha=alpha)
    return {"achieved_power": achieved_power, "target_power": target_power,
            "adequate": achieved_power >= target_power,
            "min_n_required": int(np.ceil(min_n)),
            "recommendation": "PROCEED" if achieved_power >= target_power else f"NEED_{int(np.ceil(min_n-n_samples))}_MORE_SAMPLES"}
```

### Randomization Validation

```python
def audit_randomization(treatment, control, covariates):
    """Did randomization actually balance groups?"""
    imbalances = {}
    for cov in covariates:
        t_mean, c_mean = treatment[cov].mean(), control[cov].mean()
        pooled_std = np.sqrt((treatment[cov].var() + control[cov].var()) / 2)
        d = (t_mean - c_mean) / max(pooled_std, 1e-10)
        imbalances[cov] = {"cohens_d": float(d), "balanced": abs(d) < 0.1,
                           "severity": "OK" if abs(d) < 0.1 else "IMBALANCED" if abs(d) < 0.25 else "SEVERE"}
    return imbalances
```

## Human Subject Research Data

```python
IRB_CHECKS = {
    "consent": {"documented": False, "informed": False, "voluntary": False},
    "risk_benefit": {"minimal_risk": False, "benefits_outweigh_risks": False},
    "privacy": {"de_identified": False, "re_id_risk_assessed": False},
    "vulnerable": {"children_protected": True, "prisoners_excluded": True, "cognitively_impaired_excluded": True},
}

def detect_recruitment_bias(participants, target_population_demographics):
    """Who volunteers vs who doesn't?"""
    bias = {}
    for attr, target_dist in target_population_demographics.items():
        actual_dist = participants[attr].value_counts(normalize=True).to_dict()
        for group, target_pct in target_dist.items():
            actual_pct = actual_dist.get(group, 0)
            bias[f"{attr}_{group}"] = {"target": target_pct, "actual": actual_pct,
                                        "delta": actual_pct - target_pct,
                                        "underrepresented": actual_pct < target_pct * 0.7}
    return {"biases": bias, "representative": sum(1 for b in bias.values() if b["underrepresented"]) == 0}

def audit_longitudinal_retention(participants_over_time):
    """Who drops out? Pattern detection."""
    completers = participants_over_time[-1]
    dropouts = [p for p in participants_over_time[0] if p["id"] not in set(c["id"] for c in completers)]
    
    dropout_chars = {attr: np.mean([d[attr] for d in dropouts if attr in d]) for attr in ["age", "baseline_score"]}
    completer_chars = {attr: np.mean([c[attr] for c in completers if attr in c]) for attr in ["age", "baseline_score"]}
    
    return {"retention_rate": len(completers) / len(participants_over_time[0]),
            "dropout_characteristics": dropout_chars,
            "completer_characteristics": completer_chars,
            "differential_attrition": any(abs(dropout_chars.get(k,0) - completer_chars.get(k,0)) / max(abs(completer_chars.get(k,1)), 1) > 0.15 for k in dropout_chars)}
```

## Survey / Questionnaire Data

```python
def validate_question(question_text, response_options=None):
    """Is this a well-designed survey question?"""
    issues = []
    if any(w in question_text.lower() for w in ["always", "never", "all", "none", "every"]):
        issues.append("absolute_wording")
    if "and" in question_text and "or" not in question_text:
        issues.append("double_barreled")
    if "?" not in question_text:
        issues.append("not_a_question")
    if len(question_text.split()) > 40:
        issues.append("too_complex")
    if response_options:
        if len(response_options) < 2: issues.append("single_option")
        if any(r in response_options for r in ["Other", "None of the above", "All of the above"]):
            pass  # good
    return {"valid": len(issues) == 0, "issues": issues}

def model_survey_fatigue(respondent_sessions):
    """Quality decay per question."""
    fatigue_curve = []
    for session in respondent_sessions:
        for q_idx, q_data in enumerate(session["questions"]):
            fatigue_curve.append({"position": q_idx, "response_time_sec": q_data["response_time"],
                                  "skipped": q_data.get("skipped", False)})
    # Fit decay curve
    positions = [f["position"] for f in fatigue_curve]
    times = [f["response_time_sec"] for f in fatigue_curve]
    decay_rate, _ = np.polyfit(positions, times, 1)
    return {"decay_rate_sec_per_question": float(decay_rate),
            "fatigue_warning": decay_rate > 2.0,  # >2 sec longer per question
            "recommended_max_questions": int(300 / abs(decay_rate)) if abs(decay_rate) > 0.1 else 50}

NON_RESPONSE_PATTERNS = {
    "item_nonresponse": "Don't know, prefer not to answer, skipped",
    "unit_nonresponse": "Entire survey not completed",
    "wave_nonresponse": "Missed follow-up wave",
    "breakoff": "Started but stopped before completion",
}
```

## Qualitative Data Integration

```python
def validate_coding_scheme(codes, double_coded_data, coders):
    """Is the coding scheme consistent and comprehensive?"""
    n_coders = len(coders)
    all_codes = set(codes)
    used_codes = set()
    agreement = []
    
    for item_id, item_codes in double_coded_data.items():
        for coder_a, coder_b in combinations(range(n_coders), 2):
            codes_a = set(item_codes[coder_a])
            codes_b = set(item_codes[coder_b])
            used_codes.update(codes_a, codes_b)
            # Cohen's kappa for multi-label
            overlap = len(codes_a & codes_b)
            total = len(codes_a | codes_b)
            agreement.append(overlap / max(total, 1))
    
    unused_codes = all_codes - used_codes
    return {"mean_agreement": np.mean(agreement), "kappa": float(_cohens_kappa_multilabel(double_coded_data)),
            "unused_codes": list(unused_codes), "coding_scheme_exhaustive": len(unused_codes) == 0,
            "acceptable_agreement": np.mean(agreement) > 0.7}

def detect_thematic_saturation(coded_themes_over_time, threshold=0.95):
    """When to stop collecting — no new themes emerging."""
    cumulative_unique = []
    seen = set()
    for i, new_themes in enumerate(coded_themes_over_time):
        n_before = len(seen)
        seen.update(new_themes)
        cumulative_unique.append(len(seen))
        if i >= 3 and len(seen) / cumulative_unique[i] <= threshold:
            return {"saturated_at_interview": i, "n_themes": len(seen),
                    "recommendation": "STOP_COLLECTING"}
    return {"saturated": False, "n_themes": len(seen),
            "recommendation": "CONTINUE_COLLECTING"}
```

## Quality Gate

- Hypothesis: testable, falsifiable, measurable, specific.
- Control groups: no treatment contamination, demographics balanced.
- Power analysis: achieved power ≥ 0.8 for all primary outcomes.
- IRB: consent documented, re-identification risk assessed.
- Survey: all questions pass validation, fatigue < 2s/ question, non-response bias assessed.
- Qualitative: inter-coder agreement > 0.7, thematic saturation reached.

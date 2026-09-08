# SkillPack Quality Gate Policy
# Validation Space - OPA/Rego

package skillpack.gates

default allow = false

# Minimum grade gate
allow {
    input.grade >= input.minimum_grade
}

# Score threshold gate
allow {
    input.total_score >= input.min_score
}

# Critical dimensions must pass
critical_dimensions_pass {
    input.dimensions.Security >= 80
    input.dimensions.Provenance >= 70
}

# Production readiness
production_ready {
    input.grade >= "A"
    critical_dimensions_pass
    no_error_issues
}

no_error_issues {
    count([i | input.issues[i].severity == "Error"]) == 0
}

# CI gate - fail on errors
ci_gate = result {
    result := {
        "pass": allow,
        "grade": input.grade,
        "score": input.total_score,
        "critical_pass": critical_dimensions_pass,
        "error_count": count([i | input.issues[i].severity == "Error"])
    }
}

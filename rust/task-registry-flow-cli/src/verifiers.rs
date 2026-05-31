use crate::schema::{CheckStatus, RuntimeSubject, VerifierResult, VerifierType};

pub(crate) fn verifier_result(
    behavior_id: impl Into<String>,
    verifier_type: VerifierType,
    status: CheckStatus,
    path: impl Into<String>,
    expected: impl Into<String>,
    actual: impl Into<String>,
) -> VerifierResult {
    let path = path.into();
    VerifierResult {
        behavior_id: behavior_id.into(),
        verifier_type,
        status,
        subject: RuntimeSubject::path("verifier-target", path),
        expected: expected.into(),
        actual: actual.into(),
    }
}

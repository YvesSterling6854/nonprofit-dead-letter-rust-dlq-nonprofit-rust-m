#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobKind {
    DonorReceipt,
    VolunteerReminder,
    CampaignReport,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonprofitJob {
    pub kind: JobKind,
    pub recipient: String,
    pub attempts: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobDecision {
    Process,
    DeadLetter { reason: String },
}

pub fn decide(job: &NonprofitJob, max_attempts: u32) -> JobDecision {
    if job.attempts >= max_attempts {
        JobDecision::DeadLetter { reason: format!("{} attempts reached", job.attempts) }
    } else {
        JobDecision::Process
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn poison_receipt_goes_to_dead_letter_after_cutover_limit() {
        let job = NonprofitJob {
            kind: JobKind::DonorReceipt,
            recipient: "donor-42".into(),
            attempts: 3,
        };
        assert_eq!(decide(&job, 3), JobDecision::DeadLetter {
            reason: "3 attempts reached".into(),
        });
    }
}

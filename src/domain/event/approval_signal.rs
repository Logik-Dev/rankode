use crate::domain::MediaFileId;

#[derive(Debug)]
pub enum ApprovalSignal {
    Approved {
        media_file_id: MediaFileId,
        crf: u8,
        approved_by: String,
    },
    Rejected {
        media_file_id: MediaFileId,
        rejected_by: String,
    },
}

use std::collections::HashSet;

use crate::state::NftInfo;

pub fn count_unique_values(nft_infos: Vec<NftInfo>) -> usize {
    // Sử dụng HashSet để lưu trữ các giá trị không trùng nhau
    let mut unique_values: HashSet<String> = HashSet::new();

    // Duyệt qua từng LockupTerm và thêm giá trị vào HashSet
    for nft in nft_infos.iter() {
        unique_values.insert(nft.token_id.clone());
    }

    // Trả về số lượng giá trị không trùng nhau trong HashSet
    unique_values.len()
}

pub trait scheduler {
    fn select_candidate_node(&self) -> void;
    fn score(&self) -> u64;
    fn pick(&self) -> void;
}



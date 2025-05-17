impl worker {
    pub fn collect_stats(&self) {
        println!("I will collect stats");
    }

    pub fn run_task(&self) {
        println!("I will start or stop a task");
    }

    pub fn start_task(&self) {
        println!("I will start a task");
    }

    pub fn stop_task(&self) {
        println!("I will stop a task");
    }
}
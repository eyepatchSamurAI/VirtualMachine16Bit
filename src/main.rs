use cpu_tests::test_cpu;

mod cpu;
mod create_memory;
mod instructions;
mod cpu_tests;

fn main() {
    test_cpu();
}


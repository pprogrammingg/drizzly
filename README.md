# drizzly
Rust project to showcase processing of streamed data

# Estimate
1. Read CSV in own thread, use MCSP (all STD lib) to read the CSV in batches - unit test just simply output what is read
and deserialized to vec of CSV Records 
Create dummy process_and_update client accounts in their own worker threads (30 min )

2. In-memory queue up transactions per client
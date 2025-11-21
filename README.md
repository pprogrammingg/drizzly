# drizzly
Rust project to showcase processing of streamed data

# High-LevelArchitecture
1. In own thread read CSV records in chunk, call Dispatcher thread using MCSP to call global Dispatcher thread
2. Dispatcher thread is use dispatch (via MCSP - i.e.  FIFO-based) a single transaction in a chunk of transactions 
to a worker assigned per client. We use a bounded pool of workers (based on number of cores) and map client id to 
a hash to be used as worker handle. 
This accomplishes multiple things:
a) MCSP is FIFO and each client gets dedicated worker so transactions are processed in-order for the specific client
b) bounded number of workers, means different clients can be processed in parallel, while keeping the number of threads
bounded.

   
# Estimate
1. Read CSV in own thread in chunks, deserialize records (make sure amounts are 4 decimal) send data to Dispatcher via MCSP. 
Dispatcher send records 1 by 1 via another MCSP to client specific worker threads.
Client specific worker thread print its name (deterministic hash based on client_id) and the transaction 
it is going to process (20-50 min )

2. Inside the worker thread: 
 - Check whether Client Account is locked, 
 - if locked so print error, sends error to error reporting channel
 - if not locked, perform the transaction type
   - fetch client from Arc RWLock Hashmap of Client Id vs Client State , if client does nto exist insert it
   - process transaction based on type (use a separate function per type so that it is cleaner and more readable ), if success
     print success message and write it to Client tx history
     - if process tx failed,  print error and submit it to the channel

3. Place the error reporting channel receive end in CSV reader as it is top-level, 
simply append error to a file (20min)

4. When CSV thread finishes, go through hashmap of clients and print all their balances to output (make sure amounts are 4
decimal) (20 min)

5. Additional test coverage and manual testing (30 min)
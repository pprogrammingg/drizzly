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

3. Collect and print all errors to STD Err 

4. When CSV thread finishes, go through hashmap of clients and print all their balances to output (make sure amounts are 4
decimal) 

5. Additional test coverage and manual testing (30 min)


# Improvements
CSV read can also take chunks instead of reading 1 by 1 to further enhance speed.

# Testing Examples
Input

type,client,tx,amount
deposit,1,1001,100.21466
deposit,2,1002,234.03465456
withdrawal,1,999,50.023233
withdrawal,1,9999,523.323
deposit,2,2002,10.22
dispute,2,1002,
chargeback,2,1002,
deposit,2,3002,51.23
deposit,3,1003,100
depost,3,2003,1000


Output
client,available,held,total,locked
3,100.0000,0.0000,100.0000,false
2,10.2200,0.0000,10.2200,true
1,50.1915,0.0000,50.1915,false

STD Err
- CSV deserialization error: tests/transactions.csv: CSV deserialize error: record 10 (line: 11, byte: 224): unknown variant `depost`, expected one of `deposit`, `withdrawal`, `dispute`, `resolve`, `chargeback`
- Client has insufficient balance for withdrawal: 1:9999
- Client account is frozen, cannot perform transaction. More info: client-id 2, tx-id 3002


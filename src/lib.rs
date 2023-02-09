use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

impl ThreadPool {
    /// スレッドプールを生成する
    ///
    /// * size: プール数
    ///
    /// ## Panic
    ///
    /// sizeが0ならパニックする。
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        // チャネルの送信者（sender）と受信者（receiver）を生成する
        let (sender, receiver) = mpsc::channel();

        // worker間でreceiverの所有権を共有化し、スレッド間で値を可変化する
        // Arc型は複数のワーカーに受信者を所有させ、Mutexにより、1度に受信者から1つの仕事をたった1つのワーカーが受け取ることを保証する
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            let worker = Worker::new(id, Arc::clone(&receiver));
            // 生成したworkerをworkersに挿入する
            workers.push(worker);
        }

        ThreadPool { workers, sender }
    }

    // 引数で受け取ったジョブをworkerに引き渡していることは分かるが、
    // ソースコードの中で行っていることの詳細は不明
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.send(job).unwrap();
    }
}

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv().unwrap();

            println!("Worker {} got a job: executing", id);

            job();
        });
        Worker { id, thread }
    }
}

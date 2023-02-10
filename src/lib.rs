use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

// Dropトレイトを実装することでThreadPoolがdropされる場合の動作をカスタマイズする
// ThreadPoolのdrop時にスレッドの終了を待機する
impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        // 全てのスレッドが終了するのを待つ
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            // Option型を付与することでtakeメソッドを呼び出すことにより所有権を移動できる
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
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

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    // 受け取ったジョブ（関数）をチャネルに送信している
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        // 引数として受け取ったreceiverの所有権を盗み、チャネルの受信を契機に処理を実行するクロージャ
        // なお、このクロージャは別スレッドで起動する
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();

            match message {
                // jobを受け取った場合はそのjobを実行する
                Ok(job) => {
                    println!("Worker {id} got a job: executing");

                    job();
                }
                // エラーを受け取った場合（チャネルが閉じられた場合）はloopを抜けてworkerを停止する
                Err(err) => {
                    println!("{err}"); // "receiving on a closed channel"
                    println!("Worker {id} disconnected; shutting down.");
                    break;
                }
            }
        });

        println!("worker #{} が作られました", id);

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

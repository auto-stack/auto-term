//! 014: 有界环形缓冲——满则丢最旧并计数(内存上限恒定 + 溢出预警)。
//!
//! 设计裁定:终端输出是**可损流**(丢最旧 = 画面短暂缺页,好过进程
//! 拖死系统);溢出计数器 + 最后溢出标记就是"预警"本体,glue 侧读取
//! 后打印/落报告,即可回答"为什么超限"。

use std::collections::VecDeque;

pub struct RingBuffer<T> {
    queue: VecDeque<T>,
    capacity: usize,
    /// 累计被挤掉的最旧条目个数(预警/取证)。
    dropped: u64,
}

impl<T> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            queue: VecDeque::with_capacity(capacity.min(1024)),
            capacity: capacity.max(1),
            dropped: 0,
        }
    }

    /// 入队;满则挤掉最旧(计入 dropped)并**返回被挤条目**——调用方须
    /// 据此回补流量记账(014 逐出记账 bug:pending_bytes 在逐出后无人
    /// 销账,虚高累计越过上限后 reader 永久反压休眠,终端假死)。
    pub fn push(&mut self, value: T) -> Option<T> {
        if self.queue.len() >= self.capacity {
            let evicted = self.queue.pop_front();
            self.dropped += 1;
            self.queue.push_back(value);
            evicted
        } else {
            self.queue.push_back(value);
            None
        }
    }

    /// 取走全部积压(消费侧一次排空)。
    pub fn take_all(&mut self) -> VecDeque<T> {
        std::mem::take(&mut self.queue)
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// 累计挤掉条目数。
    pub fn dropped(&self) -> u64 {
        self.dropped
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_overwrites_oldest_and_counts() {
        let mut ring = RingBuffer::new(3);
        for v in ["a", "b", "c"] {
            assert!(ring.push(v).is_none());
        }
        assert_eq!(ring.push("d"), Some("a"), "满后入队应挤掉最旧");
        assert_eq!(ring.dropped(), 1);
        let drained: Vec<&str> = ring.take_all().into_iter().collect();
        assert_eq!(drained, vec!["b", "c", "d"], "最旧的 a 被挤掉");
        assert!(ring.is_empty());
        assert_eq!(ring.dropped(), 1, "drain 不清零计数");
    }
}

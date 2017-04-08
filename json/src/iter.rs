use std::io;

pub struct LineColIterator<I> {
    iter: I,
    line: usize,
    col: usize,
}

impl<I> LineColIterator<I>
    where I: Iterator<Item = io::Result<u8>>
{
    pub fn new(iter: I) -> LineColIterator<I> {
        LineColIterator {
            iter: iter,
            line: 1,
            col: 0,
        }
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn col(&self) -> usize {
        self.col
    }
}

impl<I> Iterator for LineColIterator<I>
    where I: Iterator<Item = io::Result<u8>>
{
    type Item = io::Result<u8>;

    fn next(&mut self) -> Option<io::Result<u8>> {
        match self.iter.next() {
            None => None,
            Some(Ok(b'\n')) => {
                self.line += 1;
                self.col = 0;
                Some(Ok(b'\n'))
            }
            Some(Ok(c)) => {
                self.col += 1;
                Some(Ok(c))
            }
            Some(Err(e)) => Some(Err(e)),
        }
    }
}

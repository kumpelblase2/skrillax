use codegen::Block;

pub trait CodeContainer {
    fn attach_block<F>(&mut self, block_fn: F) -> &mut Self
    where
        F: FnOnce() -> codegen::Block;
    fn new_line<T>(&mut self, line: T) -> &mut Self
    where
        T: ToString;
}

impl CodeContainer for codegen::Function {
    fn attach_block<F>(&mut self, block_fn: F) -> &mut Self
    where
        F: FnOnce() -> Block,
    {
        self.push_block(block_fn())
    }

    fn new_line<T>(&mut self, line: T) -> &mut Self
    where
        T: ToString,
    {
        self.line(line)
    }
}

impl CodeContainer for codegen::Block {
    fn attach_block<F>(&mut self, block_fn: F) -> &mut Self
    where
        F: FnOnce() -> Block,
    {
        self.push_block(block_fn())
    }

    fn new_line<T>(&mut self, line: T) -> &mut Self
    where
        T: ToString,
    {
        self.line(line)
    }
}

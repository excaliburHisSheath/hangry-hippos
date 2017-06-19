function assert(condition, message) {
    if (!condition) {
        if (message != null) {
            throw new Error(message);
        } else {
            throw new Error('Assertion failed!');
        }
    }
}

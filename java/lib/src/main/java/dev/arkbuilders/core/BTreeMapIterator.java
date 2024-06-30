package dev.arkbuilders.core;

import java.util.Iterator;
import java.util.Map;
import java.util.NoSuchElementException;

/**
 * Represents an iterator over a BTreeMap.
 */
public class BTreeMapIterator implements Iterator<Map.Entry<String, String>>, AutoCloseable {
    private long btreemap_ptr;

    private static native long create(long file_storage_ptr);

    private static native boolean hasNext(long btreemap_ptr);

    private static native Object next(long btreemap_ptr);

    private static native void drop(long btreemap_ptr);

    BTreeMapIterator(long file_storage_ptr) {
        this.btreemap_ptr = create(file_storage_ptr);
    }

    @Override
    public boolean hasNext() {
        return hasNext(this.btreemap_ptr);
    }

    @SuppressWarnings("unchecked")
    @Override
    public Map.Entry<String, String> next() {
        if (!hasNext()) {
            throw new NoSuchElementException();
        }
        return (Map.Entry<String, String>) next(this.btreemap_ptr);
    }

    @Override
    public void close() {
        if (this.btreemap_ptr != 0) {
            drop(this.btreemap_ptr);
            this.btreemap_ptr = 0;
        }
    }
}

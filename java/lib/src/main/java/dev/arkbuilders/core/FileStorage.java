package dev.arkbuilders.core;

/**
 * Represents a file storage system that persists data to disk.
 */
public class FileStorage {
    private long fileStoragePtr;

    static {
        System.loadLibrary("fs_storage");
    }

    /**
     * Represents the synchronization status between in-memory key-value mapping and
     * external file system storage.
     */
    public enum SyncStatus {
        /**
         * No synchronization needed
         */
        InSync,
        /**
         * In-memory key-value mapping is stale
         */
        MappingStale,
        /**
         * External file system storage is stale
         */
        StorageStale,
        /**
         * In-memory key-value mapping and external file system storage diverge
         */
        Diverge,
    }

    private static native long create(String label, String path);

    private static native void set(String id, String value, long file_storage_ptr);

    private static native void remove(String id, long file_storage_ptr);

    private static native void sync(long file_storage_ptr);

    private static native SyncStatus syncStatus(long file_storage_ptr);

    private static native Object readFS(long file_storage_ptr);

    private static native String get(String id, long file_storage_ptr);

    private static native void writeFS(long file_storage_ptr);

    private static native void erase(long file_storage_ptr);

    private static native void merge(long file_storage_ptr, long other_file_storage_ptr);

    /**
     * Creates a new file storage system.
     *
     * @param label The label of the file storage system.
     * @param path  The path to the file storage system.
     */
    public FileStorage(String label, String path) {
        this.fileStoragePtr = create(label, path);
    }

    /**
     * Set a key-value pair in the internal mapping.
     *
     * @param id    The key.
     * @param value The value.
     */
    public void set(String id, String value) {
        set(id, value, this.fileStoragePtr);
    }

    /**
     * Remove an entry from the internal mapping given a key.
     *
     * @param id The key.
     */
    public void remove(String id) {
        remove(id, this.fileStoragePtr);
    }

    /**
     * Sync the in-memory storage with the storage on disk.
     */
    public void sync() {
        sync(this.fileStoragePtr);
    }

    /**
     * Compare the timestamp of the storage file with the timestamp of the in-memory
     * storage and the last written to time to determine if either of the two
     * requires syncing.
     *
     * @return The synchronization status.
     */
    public SyncStatus syncStatus() {
        return syncStatus(this.fileStoragePtr);
    }

    /**
     * Read the data from file
     *
     * @return The file storage system.
     */
    public Object readFS() {
        return readFS(this.fileStoragePtr);
    }

    /**
     * Get the value of a key from the internal mapping.
     *
     * @param id The key.
     * @return The value.
     */
    public String get(String id) {
        return get(id, this.fileStoragePtr);
    }

    /**
     * Write the data to file.
     * 
     * Note: Update the modified timestamp in file metadata to avoid OS timing
     * issues.
     * See https://github.com/ARK-Builders/ark-rust/pull/63#issuecomment-2163882227
     */
    public void writeFS() {
        writeFS(this.fileStoragePtr);
    }

    /**
     * Erase the file from disk
     */
    public void erase() {
        erase(this.fileStoragePtr);
    }

    /**
     * Merge the data from another storage instance into this storage instance
     * 
     * @param other The other storage instance
     */
    public void merge(FileStorage other) {
        merge(this.fileStoragePtr, other.fileStoragePtr);
    }

    /**
     * Create a new iterator for the BTreeMap
     * 
     * @return The iterator
     */
    public BTreeMapIterator iterator() {
        return new BTreeMapIterator(this.fileStoragePtr);
    }
}

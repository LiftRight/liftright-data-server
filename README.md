# liftright-data-server
Data collection server for gamified LiftRight study

[![Build Status](https://travis-ci.com/LiftRight/liftright-data-server.svg?branch=main)](https://travis-ci.com/LiftRight/liftright-data-server)


# Using this data

## Data Model

 There are 2 high level record schema, `Repetition` and `ImuData`
 The data is stored using the [Bucket pattern](https://www.mongodb.com/blog/post/building-with-patterns-the-bucket-pattern)

#### `ObjectId`
ObjectId is an internal Mongo data type. When represented as JSON it is an object with a single `$oid` member. For consistency a Rust style definition is provided:

```rust
pub struct ObjectId {
    pub $oid: String //a u64 in base 16
}
```

#### `Session`
A session represents a single continuous use of the application, from beginning workout to ending. The session schema has 2 fields

```rust
pub struct Session {
    pub device_id: Uuid,
    pub session_id: Uuid,
}
```

### Buckets at a high level
Buckets provide a way to maximize write performance. LiftRight writes a large amount of data into MongoDb. Buckets are defined generically in `query_selector.rs`. They allow an arbitrary structure to be serialized and updated using the bucket pattern.


#### `ImuData`
There are a series of schemas for ImuData, they are nested.
The schemas for these records are defined in `imurecords.rs`.


##### `ImuRecordPair`
An `ImuRecordPair` is a pair of `ImuRecord`s, an accelerometer record and a gyroscope record.
```rust
pub struct ImuRecordPair {
    pub acc: ImuRecord,
    pub gyro: ImuRecord,
}
```

##### `ImuRecord`
Represents an individual IMU datapoint from either the accelerometer and the gyroscope.

The `time` member of this data does not represent any meaningful point in absolute time. TThe values are relative to the device generating them. They have nanosecond precision and can be used for ordering and computing duration only.

```rust
pub struct ImuRecord {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub time: i64,
}
```

##### The ImuData bucket
This struct is not strictly defined in the code. It is built dynamically by the `to_bucket_update` method in `query_selector.rs`.

Unlike the `time` member of the `ImuRecord` struct, the timestamp appended onto the `id` member has a Wall time representable value. The value is in nanoseconds, but it can be read into most languages for analytics by convertng to milliseconds (see your preferred time library for API documentation).

```rust
pub struct ImuRecordBucket {
    pub _id: ObjectId
    pub device_id: Uuid,
    pub kind: "ImuData",
    pub session_id: Uuid,
    pub id: [device_id]_[timestamp of last insert(ns)],
    pub imu_data: Vec<ImuRecordPair>
    pub imu_record_count: u64
}
```

The data stored in the `imu_data` member is stored in order, oldest first.


#### Repetitions and Sets
Sets are stored as discreet objects. Bucket size is much smaller for Repetition data.
the time member of `Repetition` is also nanosecond precision and not wall-time meaningful. It can be used for determining intervals and ordering. If a wall time meaningful time for an individual Repetition is needed, the timestamp of the set id can be used for the last repetitionand offsets can be computed. 

##### `Repetition`
```rust
pub struct Repetition {
    number: i32,
    rom: f64,
    duration: f64,
    time: i64,
}
```


##### The Repetition bucket
Similar to the ImuData bucket, this record is not defined in the code.

```rust
pub struct RepetitionBucket {
    pub _id: ObjectId,
    pub device_id: Uuid,
    pub kind: "Repetition",
    pub session_id: Uuid,
    pub set_id: Uuid,
    pub id: [device_id]_[timestamp of last insert(ns)],
    pub rep_count: u64,
    pub repetitions: Vec<Repetition>
}
```


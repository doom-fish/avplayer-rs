import AVPlayerABI
import Foundation

indirect enum AVPBinaryValue {
    case null
    case bool(Bool)
    case signed(Int64)
    case unsigned(UInt64)
    case real(Double)
    case string(String)
    case data([UInt8])
    case array([AVPBinaryValue])
    case dictionary([String: AVPBinaryValue])
    case object(UnsafeMutableRawPointer)

    func field<T: AVPBridge>(_ index: Int, as type: T.Type = T.self) throws -> T {
        guard case .array(let values) = self else {
            throw BridgeError.message("invalid binary bridge record")
        }
        return try T(binaryValue: index < values.count ? values[index] : .null)
    }
}

protocol AVPBridge {
    var binaryValue: AVPBinaryValue { get }
    init(binaryValue: AVPBinaryValue) throws
}

extension AVPBinaryValue: AVPBridge {
    var binaryValue: AVPBinaryValue { self }

    init(binaryValue: AVPBinaryValue) {
        self = binaryValue
    }
}

extension String: AVPBridge {
    var binaryValue: AVPBinaryValue { .string(self) }

    init(binaryValue: AVPBinaryValue) throws {
        guard case .string(let value) = binaryValue else {
            throw BridgeError.message("invalid binary bridge string")
        }
        self = value
    }

    init(bridge pointer: UnsafePointer<AVPValue>) {
        self = (try? avpDecodeValue(pointer, as: String.self)) ?? ""
    }

    func withBridgeString<T>(_ body: (UnsafePointer<AVPValue>) -> T) -> T {
        let pointer = ffiString(self)!
        defer { avp_value_free(pointer) }
        return body(UnsafePointer(pointer))
    }
}

extension Bool: AVPBridge {
    var binaryValue: AVPBinaryValue { .bool(self) }

    init(binaryValue: AVPBinaryValue) throws {
        guard case .bool(let value) = binaryValue else {
            throw BridgeError.message("invalid binary bridge boolean")
        }
        self = value
    }
}

protocol AVPBridgeInteger: FixedWidthInteger, AVPBridge {}

extension AVPBridgeInteger {
    var binaryValue: AVPBinaryValue {
        Self.isSigned ? .signed(Int64(self)) : .unsigned(UInt64(self))
    }

    init(binaryValue: AVPBinaryValue) throws {
        let value: Self?
        switch binaryValue {
        case .signed(let raw):
            value = Self(exactly: raw)
        case .unsigned(let raw):
            value = Self(exactly: raw)
        default:
            value = nil
        }
        guard let value else {
            throw BridgeError.message("invalid binary bridge integer")
        }
        self = value
    }
}

extension Int: AVPBridgeInteger {}
extension Int32: AVPBridgeInteger {}
extension Int64: AVPBridgeInteger {}
extension UInt: AVPBridgeInteger {}
extension UInt8: AVPBridgeInteger {}
extension UInt32: AVPBridgeInteger {}
extension UInt64: AVPBridgeInteger {}

extension Double: AVPBridge {
    var binaryValue: AVPBinaryValue { .real(self) }

    init(binaryValue: AVPBinaryValue) throws {
        switch binaryValue {
        case .real(let value):
            self = value
        case .signed(let value):
            self = Double(value)
        case .unsigned(let value):
            self = Double(value)
        default:
            throw BridgeError.message("invalid binary bridge number")
        }
    }
}

extension Float: AVPBridge {
    var binaryValue: AVPBinaryValue { .real(Double(self)) }

    init(binaryValue: AVPBinaryValue) throws {
        self = Float(try Double(binaryValue: binaryValue))
    }
}

extension Optional: AVPBridge where Wrapped: AVPBridge {
    var binaryValue: AVPBinaryValue { map(\.binaryValue) ?? .null }

    init(binaryValue: AVPBinaryValue) throws {
        if case .null = binaryValue {
            self = nil
        } else {
            self = try Wrapped(binaryValue: binaryValue)
        }
    }
}

extension Array: AVPBridge where Element: AVPBridge {
    var binaryValue: AVPBinaryValue {
        if Element.self == UInt8.self, let bytes = self as? [UInt8] {
            return .data(bytes)
        }
        return .array(map(\.binaryValue))
    }

    init(binaryValue: AVPBinaryValue) throws {
        if Element.self == UInt8.self, case .data(let bytes) = binaryValue,
           let values = bytes as? [Element] {
            self = values
            return
        }
        guard case .array(let values) = binaryValue else {
            throw BridgeError.message("invalid binary bridge array")
        }
        self = try values.map(Element.init(binaryValue:))
    }
}

extension Dictionary: AVPBridge where Key == String, Value: AVPBridge {
    var binaryValue: AVPBinaryValue { .dictionary(mapValues(\.binaryValue)) }

    init(binaryValue: AVPBinaryValue) throws {
        guard case .dictionary(let values) = binaryValue else {
            throw BridgeError.message("invalid binary bridge dictionary")
        }
        self = try values.mapValues(Value.init(binaryValue:))
    }
}

struct AVPObjectHandle: AVPBridge {
    let pointer: UnsafeMutableRawPointer

    var binaryValue: AVPBinaryValue { .object(pointer) }

    init(pointer: UnsafeMutableRawPointer) {
        self.pointer = pointer
    }

    init(binaryValue: AVPBinaryValue) throws {
        guard case .object(let pointer) = binaryValue else {
            throw BridgeError.message("invalid binary bridge object")
        }
        self.pointer = pointer
    }
}

private final class AVPValueArena {
    private var allocations: [() -> Void] = []

    deinit {
        for release in allocations.reversed() {
            release()
        }
    }

    func allocate<T>(_ values: [T]) -> UnsafePointer<T>? {
        guard !values.isEmpty else { return nil }
        let pointer = UnsafeMutablePointer<T>.allocate(capacity: values.count)
        _ = pointer.initialize(from: values, count: values.count)
        allocations.append {
            pointer.deinitialize(count: values.count)
            pointer.deallocate()
        }
        return UnsafePointer(pointer)
    }

    func bytes(_ bytes: [UInt8]) -> AVPBytes {
        AVPBytes(data: allocate(bytes), length: bytes.count)
    }

    func store(_ value: AVPBinaryValue, depth: Int) throws -> AVPValue {
        guard depth <= 128 else {
            throw BridgeError.message("binary bridge value exceeds the depth limit")
        }
        var raw = AVPValue()
        switch value {
        case .null:
            raw.kind = 0
        case .bool(let value):
            raw.kind = 1
            raw.unsigned_value = value ? 1 : 0
        case .signed(let value):
            raw.kind = 2
            raw.signed_value = value
        case .unsigned(let value):
            raw.kind = 3
            raw.unsigned_value = value
        case .real(let value):
            raw.kind = 4
            raw.real_value = value
        case .string(let value):
            raw.kind = 5
            raw.bytes = bytes(Array(value.utf8))
        case .data(let value):
            raw.kind = 6
            raw.bytes = bytes(value)
        case .array(let values):
            raw.kind = 7
            raw.values = allocate(try values.map { try store($0, depth: depth + 1) })
            raw.count = values.count
        case .dictionary(let values):
            raw.kind = 8
            let keys = values.keys.sorted()
            raw.keys = allocate(keys.map { bytes(Array($0.utf8)) })
            raw.values = allocate(try keys.map { try store(values[$0]!, depth: depth + 1) })
            raw.count = keys.count
        case .object(let pointer):
            raw.kind = 9
            raw.object = pointer
        }
        return raw
    }
}

func avpEncodeValue<T: AVPBridge>(_ value: T) throws -> UnsafeMutablePointer<AVPValue> {
    let arena = AVPValueArena()
    var raw = try arena.store(value.binaryValue, depth: 0)
    raw.owner = Unmanaged.passRetained(arena).toOpaque()
    return UnsafeMutablePointer(mutating: arena.allocate([raw])!)
}

@_cdecl("avp_value_free")
public func avp_value_free(_ pointer: UnsafeMutablePointer<AVPValue>?) {
    guard let owner = pointer?.pointee.owner else { return }
    Unmanaged<AVPValueArena>.fromOpaque(owner).release()
}

private func avpReadBytes(_ bytes: AVPBytes) throws -> [UInt8] {
    guard bytes.length > 0 else { return [] }
    guard let pointer = bytes.data else {
        throw BridgeError.message("missing binary bridge bytes")
    }
    return Array(UnsafeBufferPointer(start: pointer, count: bytes.length))
}

private func avpReadValue(_ pointer: UnsafePointer<AVPValue>, depth: Int) throws -> AVPBinaryValue {
    guard depth <= 128 else {
        throw BridgeError.message("binary bridge value exceeds the depth limit")
    }
    let raw = pointer.pointee
    switch raw.kind {
    case 0:
        return .null
    case 1 where raw.unsigned_value <= 1:
        return .bool(raw.unsigned_value != 0)
    case 2:
        return .signed(raw.signed_value)
    case 3:
        return .unsigned(raw.unsigned_value)
    case 4:
        return .real(raw.real_value)
    case 5:
        guard let string = String(bytes: try avpReadBytes(raw.bytes), encoding: .utf8) else {
            throw BridgeError.message("invalid UTF-8 bridge string")
        }
        return .string(string)
    case 6:
        return .data(try avpReadBytes(raw.bytes))
    case 7, 8:
        guard raw.count >= 0, raw.count == 0 || raw.values != nil else {
            throw BridgeError.message("missing binary bridge array")
        }
        let values = try (0..<raw.count).map {
            try avpReadValue(raw.values!.advanced(by: $0), depth: depth + 1)
        }
        if raw.kind == 7 {
            return .array(values)
        }
        guard raw.count == 0 || raw.keys != nil else {
            throw BridgeError.message("missing binary bridge keys")
        }
        var dictionary: [String: AVPBinaryValue] = [:]
        for index in 0..<raw.count {
            guard let key = String(bytes: try avpReadBytes(raw.keys![index]), encoding: .utf8) else {
                throw BridgeError.message("invalid UTF-8 bridge key")
            }
            dictionary[key] = values[index]
        }
        return .dictionary(dictionary)
    case 9:
        guard let pointer = raw.object else {
            throw BridgeError.message("missing binary bridge object")
        }
        return .object(pointer)
    default:
        throw BridgeError.message("invalid binary bridge value")
    }
}

func avpDecodeValue<T: AVPBridge>(_ pointer: UnsafePointer<AVPValue>?, as type: T.Type) throws -> T {
    guard let pointer else {
        throw BridgeError.message("missing binary bridge value")
    }
    return try T(binaryValue: avpReadValue(pointer, depth: 0))
}

func avpPropertyValue(_ object: Any?) -> AVPBinaryValue? {
    guard let object else { return nil }
    switch object {
    case is NSNull:
        return .null
    case let string as String:
        return .string(string)
    case let number as NSNumber:
        if CFGetTypeID(number) == CFBooleanGetTypeID() {
            return .bool(number.boolValue)
        }
        switch String(cString: number.objCType) {
        case "f", "d":
            return .real(number.doubleValue)
        case "C", "S", "I", "L", "Q":
            return .unsigned(number.uint64Value)
        default:
            return .signed(number.int64Value)
        }
    case let data as Data:
        return .data(Array(data))
    case let values as [Any]:
        let encoded = values.compactMap(avpPropertyValue)
        return encoded.count == values.count ? .array(encoded) : nil
    case let values as [String: Any]:
        let encoded = values.compactMapValues(avpPropertyValue)
        return encoded.count == values.count ? .dictionary(encoded) : nil
    default:
        return nil
    }
}

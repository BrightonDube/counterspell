// Counterspell C++ driver, protocol version 1 (see languages/protocol.md).
//
// The driver loads the user's solution once. It then reads one call per stdin line and
// writes one answer line per call to stdout. It never prints a verdict, an expected
// value, a timing or a complexity class: the trusted judge owns all of those. Diagnostics
// go to stderr.
//
// The sandbox builds this as `g++ -O2 -std=c++20 -o /box/out/main driver.cpp`, so this
// file `#include`s `solution.cpp` at the bottom and runs the same loop for
// `--mode bench`; the judge does all timing.
//
// Solution convention: `solution.cpp` defines
//     JSONValue counterspell_call(const std::vector<JSONValue>& args);
// which converts each argument with `from_json<T>`, calls the user's typed function,
// and wraps the result with `to_json`. Protocol types map to C++ types as: int ->
// long long, float -> double, bool -> bool, string -> std::string, list ->
// std::vector<T> of the element's mapped type, tuple -> a fixed-length
// std::vector<JSONValue> per the signature, null -> JSONValue{}. The helpers never
// coerce kinds; the driver validates each value against the signature anyway.

#include <cmath>
#include <cstdio>
#include <cstring>
#include <fcntl.h>
#include <fstream>
#include <iostream>
#include <optional>
#include <sstream>
#include <stdexcept>
#include <string>
#include <utility>
#include <variant>
#include <vector>
#include <unistd.h>

// ------------------------------------------------------------------
// JSONValue: the protocol's JSON-safe value space.
// ------------------------------------------------------------------

struct JSONValue;
using JSONList = std::vector<JSONValue>;
using JSONObject = std::vector<std::pair<std::string, JSONValue>>;

struct JSONValue {
    std::variant<std::nullptr_t, long long, double, bool, std::string, JSONList, JSONObject> value;
    JSONValue() : value(nullptr) {}
    JSONValue(std::nullptr_t) : value(nullptr) {}
    JSONValue(long long v) : value(v) {}
    JSONValue(int v) : value(static_cast<long long>(v)) {}
    JSONValue(std::size_t v) : value(static_cast<long long>(v)) {}
    JSONValue(double v) : value(v) {}
    JSONValue(bool v) : value(v) {}
    JSONValue(const std::string& v) : value(v) {}
    JSONValue(const char* v) : value(std::string(v)) {}
    JSONValue(const JSONList& v) : value(v) {}
    JSONValue(const JSONObject& v) : value(v) {}
    template <typename T> JSONValue(const std::vector<T>& v) {
        JSONList out;
        out.reserve(v.size());
        for (const T& item : v) out.emplace_back(item);
        value = std::move(out);
    }

    static const JSONValue* member(const JSONValue& object, const std::string& key) {
        auto* members = std::get_if<JSONObject>(&object.value);
        if (!members) return nullptr;
        for (const auto& [name, value] : *members) {
            if (name == key) return &value;
        }
        return nullptr;
    }
};

// Conversions the solution's shim uses. They throw std::runtime_error on a kind
// mismatch; the driver validates args against the signature first, so a mismatch
// here is a protocol-level surprise, not user data to trust.
namespace detail {
template <typename T> T convert(const JSONValue& v) {
    if constexpr (std::is_same_v<T, long long>) {
        if (auto* p = std::get_if<long long>(&v.value)) return *p;
        throw std::runtime_error("argument is not an int");
    } else if constexpr (std::is_same_v<T, double>) {
        if (auto* p = std::get_if<double>(&v.value)) return *p;
        if (auto* p = std::get_if<long long>(&v.value)) return static_cast<double>(*p);
        throw std::runtime_error("argument is not a float");
    } else if constexpr (std::is_same_v<T, bool>) {
        if (auto* p = std::get_if<bool>(&v.value)) return *p;
        throw std::runtime_error("argument is not a bool");
    } else if constexpr (std::is_same_v<T, std::string>) {
        if (auto* p = std::get_if<std::string>(&v.value)) return *p;
        throw std::runtime_error("argument is not a string");
    } else if constexpr (std::is_same_v<T, JSONValue>) {
        return v;
    } else {
        static_assert(!sizeof(T*), "unsupported from_json type");
    }
}
} // namespace detail

namespace detail {
template <typename T> struct is_std_vector : std::false_type {};
template <typename T> struct is_std_vector<std::vector<T>> : std::true_type {};
} // namespace detail

template <typename T> T from_json(const JSONValue& v) {
    if constexpr (std::is_same_v<T, JSONValue>) {
        return v;
    } else if constexpr (detail::is_std_vector<T>::value) {
        auto* list = std::get_if<JSONList>(&v.value);
        if (!list) throw std::runtime_error("argument is not a list");
        T out;
        out.reserve(list->size());
        for (const JSONValue& item : *list) out.push_back(from_json<typename T::value_type>(item));
        return out;
    } else {
        return detail::convert<T>(v);
    }
}

inline JSONValue to_json(const JSONValue& v) { return v; }
template <typename T> JSONValue to_json(const T& v) { return JSONValue(v); }

// ------------------------------------------------------------------
// Minimal JSON reader/writer: protocol v1 needs only JSON-safe values.
// ------------------------------------------------------------------

class DriverFailure : public std::runtime_error {
  public:
    explicit DriverFailure(const std::string& why, int exit = 2) : std::runtime_error(why), exitCode(exit) {}
    int exitCode;
};

class JsonReader {
  public:
    explicit JsonReader(std::string_view text) : text_(text) {}

    JSONValue parse() {
        JSONValue value = readValue();
        skipSpace();
        if (pos_ != text_.size()) throw DriverFailure("trailing data after the JSON value");
        return value;
    }

  private:
    std::string_view text_;
    std::size_t pos_ = 0;

    void skipSpace() {
        while (pos_ < text_.size() &&
               (text_[pos_] == ' ' || text_[pos_] == '\t' || text_[pos_] == '\n' ||
                text_[pos_] == '\r')) {
            pos_++;
        }
    }

    char peek() {
        if (pos_ >= text_.size()) throw DriverFailure("unexpected end of JSON");
        return text_[pos_];
    }

    void expect(char c) {
        if (peek() != c) throw DriverFailure(std::string("expected '") + c + "' in JSON");
        pos_++;
    }

    JSONValue readValue() {
        skipSpace();
        char c = peek();
        if (c == '{') return readObject();
        if (c == '[') return readArray();
        if (c == '"') return JSONValue(readString());
        if (c == 't') return readLiteral("true", JSONValue(true));
        if (c == 'f') return readLiteral("false", JSONValue(false));
        if (c == 'n') return readLiteral("null", JSONValue(nullptr));
        return readNumber();
    }

    JSONValue readLiteral(const char* word, JSONValue value) {
        std::size_t n = std::strlen(word);
        if (text_.substr(pos_, n) != std::string_view(word, n)) throw DriverFailure("bad JSON literal");
        pos_ += n;
        return value;
    }

    JSONValue readObject() {
        expect('{');
        JSONObject out;
        skipSpace();
        if (peek() == '}') {
            pos_++;
            return JSONValue(out);
        }
        while (true) {
            skipSpace();
            std::string key = readString();
            skipSpace();
            expect(':');
            out.emplace_back(std::move(key), readValue());
            skipSpace();
            if (peek() == '}') {
                pos_++;
                return JSONValue(out);
            }
            expect(',');
        }
    }

    JSONValue readArray() {
        expect('[');
        JSONList out;
        skipSpace();
        if (peek() == ']') {
            pos_++;
            return JSONValue(std::move(out));
        }
        while (true) {
            out.push_back(readValue());
            skipSpace();
            if (peek() == ']') {
                pos_++;
                return JSONValue(std::move(out));
            }
            expect(',');
        }
    }

    std::string readString() {
        expect('"');
        std::string out;
        while (true) {
            char c = peek();
            pos_++;
            if (c == '"') return out;
            if (c != '\\') {
                out += c;
                continue;
            }
            char esc = peek();
            pos_++;
            switch (esc) {
            case '"': out += '"'; break;
            case '\\': out += '\\'; break;
            case '/': out += '/'; break;
            case 'b': out += '\b'; break;
            case 'f': out += '\f'; break;
            case 'n': out += '\n'; break;
            case 'r': out += '\r'; break;
            case 't': out += '\t'; break;
            case 'u': {
                if (pos_ + 4 > text_.size()) throw DriverFailure("truncated \\u escape");
                unsigned code;
                try {
                    code = std::stoul(std::string(text_.substr(pos_, 4)), nullptr, 16);
                } catch (...) {
                    throw DriverFailure("bad \\u escape");
                }
                pos_ += 4;
                if (code >= 0xD800 && code <= 0xDBFF && text_.substr(pos_, 2) == "\\u") {
                    unsigned low = 0;
                    try {
                        low = std::stoul(std::string(text_.substr(pos_ + 2, 4)), nullptr, 16);
                    } catch (...) {
                        low = 0;
                    }
                    if (low >= 0xDC00 && low <= 0xDFFF) {
                        code = 0x10000 + ((code - 0xD800) << 10) + (low - 0xDC00);
                        pos_ += 6;
                    }
                }
                if (code < 0x80) {
                    out += static_cast<char>(code);
                } else if (code < 0x800) {
                    out += static_cast<char>(0xC0 | (code >> 6));
                    out += static_cast<char>(0x80 | (code & 0x3F));
                } else if (code < 0x10000) {
                    out += static_cast<char>(0xE0 | (code >> 12));
                    out += static_cast<char>(0x80 | ((code >> 6) & 0x3F));
                    out += static_cast<char>(0x80 | (code & 0x3F));
                } else {
                    out += static_cast<char>(0xF0 | (code >> 18));
                    out += static_cast<char>(0x80 | ((code >> 12) & 0x3F));
                    out += static_cast<char>(0x80 | ((code >> 6) & 0x3F));
                    out += static_cast<char>(0x80 | (code & 0x3F));
                }
                break;
            }
            default:
                throw DriverFailure(std::string("bad string escape \\") + esc);
            }
        }
    }

    JSONValue readNumber() {
        std::size_t start = pos_;
        if (peek() == '-') pos_++;
        while (pos_ < text_.size() && isdigit(text_[pos_])) pos_++;
        bool isFloat = false;
        if (pos_ < text_.size() && text_[pos_] == '.') {
            isFloat = true;
            pos_++;
            while (pos_ < text_.size() && isdigit(text_[pos_])) pos_++;
        }
        if (pos_ < text_.size() && (text_[pos_] == 'e' || text_[pos_] == 'E')) {
            isFloat = true;
            pos_++;
            if (pos_ < text_.size() && (text_[pos_] == '+' || text_[pos_] == '-')) pos_++;
            while (pos_ < text_.size() && isdigit(text_[pos_])) pos_++;
        }
        if (pos_ == start || (pos_ == start + 1 && text_[start] == '-')) {
            throw DriverFailure("bad JSON number");
        }
        std::string digits(text_.substr(start, pos_ - start));
        try {
            if (isFloat) {
                double d = std::stod(digits);
                if (!std::isfinite(d)) throw DriverFailure("non-finite float in JSON");
                return JSONValue(d);
            }
            return JSONValue(static_cast<long long>(std::stoll(digits)));
        } catch (const std::out_of_range&) {
            try {
                double d = std::stod(digits);
                if (std::isfinite(d)) return JSONValue(d);
            } catch (...) {
            }
            throw DriverFailure("bad JSON number " + digits);
        } catch (const std::invalid_argument&) {
            throw DriverFailure("bad JSON number " + digits);
        }
    }
};

static void appendJson(std::string& out, const JSONValue& v) {
    if (std::holds_alternative<std::nullptr_t>(v.value)) {
        out += "null";
    } else if (auto* p = std::get_if<long long>(&v.value)) {
        out += std::to_string(*p);
    } else if (auto* p = std::get_if<double>(&v.value)) {
        if (!std::isfinite(*p)) throw DriverFailure("the return value has a non-finite float");
        char buf[64];
        std::snprintf(buf, sizeof(buf), "%.17g", *p);
        out += buf;
    } else if (auto* p = std::get_if<bool>(&v.value)) {
        out += *p ? "true" : "false";
    } else if (auto* p = std::get_if<std::string>(&v.value)) {
        out += '"';
        for (unsigned char c : *p) {
            switch (c) {
            case '"': out += "\\\""; break;
            case '\\': out += "\\\\"; break;
            case '\b': out += "\\b"; break;
            case '\f': out += "\\f"; break;
            case '\n': out += "\\n"; break;
            case '\r': out += "\\r"; break;
            case '\t': out += "\\t"; break;
            default:
                if (c < 0x20) {
                    char buf[8];
                    std::snprintf(buf, sizeof(buf), "\\u%04x", c);
                    out += buf;
                } else {
                    out += static_cast<char>(c);
                }
            }
        }
        out += '"';
    } else if (auto* p = std::get_if<JSONList>(&v.value)) {
        out += '[';
        for (std::size_t i = 0; i < p->size(); i++) {
            if (i > 0) out += ',';
            appendJson(out, (*p)[i]);
        }
        out += ']';
    } else {
        // JSONObject is driver-input only; it can never be a return value.
        throw DriverFailure("the return value is not JSON-safe");
    }
}

// ------------------------------------------------------------------
// Signature descriptor.
// ------------------------------------------------------------------

struct TypeDescriptor {
    std::string kind;
    bool nullable = false;
    std::vector<TypeDescriptor> items;
};

struct Signature {
    std::string function;
    std::vector<TypeDescriptor> parameters;
    TypeDescriptor returns;
};

static bool knownKind(const std::string& kind) {
    return kind == "int" || kind == "float" || kind == "bool" || kind == "string" ||
           kind == "list" || kind == "tuple" || kind == "null";
}

static TypeDescriptor checkType(const JSONValue& raw) {
    auto* kind = JSONValue::member(raw, "kind");
    if (!kind || !std::holds_alternative<std::string>(kind->value) ||
        !knownKind(std::get<std::string>(kind->value))) {
        throw DriverFailure("a type in the signature is missing a known kind");
    }
    TypeDescriptor out;
    out.kind = std::get<std::string>(kind->value);
    if (auto* nullable = JSONValue::member(raw, "nullable")) {
        if (!std::holds_alternative<bool>(nullable->value)) {
            throw DriverFailure("the nullable flag must be true or false");
        }
        out.nullable = std::get<bool>(nullable->value);
    }
    if (out.kind == "list") {
        auto* items = JSONValue::member(raw, "items");
        if (!items) throw DriverFailure("a list type needs items");
        out.items.push_back(checkType(*items));
    } else if (out.kind == "tuple") {
        auto* items = JSONValue::member(raw, "items");
        if (!items || !std::holds_alternative<JSONList>(items->value)) {
            throw DriverFailure("a tuple type needs a list of item types");
        }
        for (const JSONValue& item : std::get<JSONList>(items->value)) {
            out.items.push_back(checkType(item));
        }
    }
    return out;
}

static Signature loadSignature() {
    const char* env = std::getenv("COUNTER_SPELL_SIGNATURE");
    std::string path = env ? env : "/box/signature.json";
    std::ifstream file(path, std::ios::binary);
    if (!file) throw DriverFailure("cannot read the signature");
    std::ostringstream text;
    text << file.rdbuf();
    JSONValue data = JsonReader(text.str()).parse();
    auto* version = JSONValue::member(data, "protocol_version");
    if (!version || !std::holds_alternative<long long>(version->value) ||
        std::get<long long>(version->value) != 1) {
        throw DriverFailure("the signature has an unsupported protocol version");
    }
    auto* function = JSONValue::member(data, "function");
    if (!function || !std::holds_alternative<std::string>(function->value)) {
        throw DriverFailure("the signature has no valid function name");
    }
    const std::string& name = std::get<std::string>(function->value);
    if (name.empty() || !(isalpha(name[0]) || name[0] == '_' || name[0] == '$')) {
        throw DriverFailure("the signature has no valid function name");
    }
    for (char c : name) {
        if (!(isalnum(c) || c == '_' || c == '$')) {
            throw DriverFailure("the signature has no valid function name");
        }
    }
    Signature sig;
    sig.function = name;
    auto* parameters = JSONValue::member(data, "parameters");
    if (!parameters || !std::holds_alternative<JSONList>(parameters->value)) {
        throw DriverFailure("the signature has no parameter list");
    }
    for (const JSONValue& parameter : std::get<JSONList>(parameters->value)) {
        auto* type = JSONValue::member(parameter, "type");
        if (!type) throw DriverFailure("a parameter in the signature is not an object");
        sig.parameters.push_back(checkType(*type));
    }
    auto* returns = JSONValue::member(data, "returns");
    if (!returns) throw DriverFailure("the signature has no return type");
    sig.returns = checkType(*returns);
    return sig;
}

// ------------------------------------------------------------------
// Conformance: does a JSON value match a descriptor? Never converts.
// ------------------------------------------------------------------

static bool conforms(const JSONValue& value, const TypeDescriptor& descriptor) {
    if (std::holds_alternative<std::nullptr_t>(value.value)) {
        return descriptor.kind == "null" || descriptor.nullable;
    }
    if (descriptor.kind == "int") {
        // The same exact-integer range as the Python and JavaScript drivers (2^53 - 1).
        constexpr long long max_safe_integer = (1LL << 53) - 1;
        auto* p = std::get_if<long long>(&value.value);
        return p && *p >= -max_safe_integer && *p <= max_safe_integer;
    }
    if (descriptor.kind == "float") {
        auto* p = std::get_if<double>(&value.value);
        return p && std::isfinite(*p);
    }
    if (descriptor.kind == "bool") return std::holds_alternative<bool>(value.value);
    if (descriptor.kind == "string") return std::holds_alternative<std::string>(value.value);
    if (descriptor.kind == "list") {
        auto* list = std::get_if<JSONList>(&value.value);
        if (!list) return false;
        for (const JSONValue& item : *list) {
            if (!conforms(item, descriptor.items[0])) return false;
        }
        return true;
    }
    if (descriptor.kind == "tuple") {
        auto* list = std::get_if<JSONList>(&value.value);
        if (!list || list->size() != descriptor.items.size()) return false;
        for (std::size_t i = 0; i < list->size(); i++) {
            if (!conforms((*list)[i], descriptor.items[i])) return false;
        }
        return true;
    }
    return false;
}

// ------------------------------------------------------------------
// The solution's entry point, provided by solution.cpp.
// ------------------------------------------------------------------

JSONValue counterspell_call(const std::vector<JSONValue>& args);

#include "solution.cpp"

// ------------------------------------------------------------------
// Call loop.
// ------------------------------------------------------------------

static void diagnose(const std::string& message) {
    std::cerr << "driver: " << message << "\n";
}

struct Call {
    long long id;
    JSONList args;
};

static Call parseCall(const std::string& line, const Signature& signature) {
    JSONValue data = JsonReader(line).parse();
    auto* t = JSONValue::member(data, "t");
    if (!t || !std::holds_alternative<long long>(t->value) || std::get<long long>(t->value) < 0) {
        throw DriverFailure("a call line has no valid call id");
    }
    auto* args = JSONValue::member(data, "args");
    if (!args || !std::holds_alternative<JSONList>(args->value)) {
        throw DriverFailure("a call line has no argument list");
    }
    const JSONList& argList = std::get<JSONList>(args->value);
    if (argList.size() != signature.parameters.size()) {
        throw DriverFailure("a call line has the wrong argument count");
    }
    for (std::size_t i = 0; i < argList.size(); i++) {
        if (!conforms(argList[i], signature.parameters[i])) {
            throw DriverFailure("an argument does not match the signature");
        }
    }
    return Call{std::get<long long>(t->value), argList};
}

// Sends fd 1 to /dev/null while user code runs so its prints cannot corrupt the
// answer stream, then restores it.
class Quiet {
  public:
    Quiet() {
        std::fflush(stdout);
        std::cout.flush();
        saved_ = dup(STDOUT_FILENO);
        int devnull = open("/dev/null", O_WRONLY);
        if (saved_ >= 0 && devnull >= 0) {
            dup2(devnull, STDOUT_FILENO);
        }
        if (devnull >= 0) close(devnull);
    }
    ~Quiet() {
        std::fflush(stdout);
        std::cout.flush();
        if (saved_ >= 0) {
            dup2(saved_, STDOUT_FILENO);
            close(saved_);
        }
    }

  private:
    int saved_ = -1;
};

int main(int argc, char** argv) {
    std::string mode = "normal";
    for (int i = 1; i < argc; i++) {
        if (std::string(argv[i]) == "--mode" && i + 1 < argc) {
            mode = argv[++i];
        }
    }
    if (mode != "normal" && mode != "bench") {
        diagnose("unknown mode " + mode);
        return 2;
    }
    try {
        Signature signature = loadSignature();
        std::string line;
        while (std::getline(std::cin, line)) {
            if (line.find_first_not_of(" \t\r") == std::string::npos) {
                throw DriverFailure("a blank call line is not valid protocol");
            }
            Call call = parseCall(line, signature);
            JSONValue result;
            try {
                Quiet quiet;
                result = counterspell_call(call.args);
            } catch (const std::exception& e) {
                diagnose(std::string("the solution raised an exception: ") + e.what());
                return 1;
            } catch (...) {
                diagnose("the solution raised an exception");
                return 1;
            }
            if (!conforms(result, signature.returns)) {
                throw DriverFailure("the return value is not JSON-safe for the signature");
            }
            std::string answer;
            answer += "{\"t\":";
            answer += std::to_string(call.id);
            answer += ",\"out\":";
            appendJson(answer, result);
            answer += "}";
            std::cout << answer << "\n" << std::flush;
        }
    } catch (const DriverFailure& failure) {
        diagnose(failure.what());
        return failure.exitCode;
    } catch (const std::exception& e) {
        diagnose(e.what());
        return 2;
    }
    return 0;
}

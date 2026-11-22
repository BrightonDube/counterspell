import java.io.BufferedReader;
import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.InputStreamReader;
import java.io.OutputStreamWriter;
import java.io.PrintStream;
import java.io.PrintWriter;
import java.lang.reflect.Array;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

/**
 * Counterspell Java driver, protocol version 1 (see languages/protocol.md).
 *
 * The driver loads the user's `Solution` class once. It then reads one call per stdin line
 * and writes one answer line per call to stdout. It never prints a verdict, an expected
 * value, a timing or a complexity class: the trusted judge owns all of those. Diagnostics
 * go to stderr.
 *
 * The sandbox runs this as `java -Xss16m -Xmx256m -XX:+UseSerialGC -cp /box/out Driver`
 * after `javac -d /box/out Solution.java Driver.java`. `--mode bench` is accepted and uses
 * the same loop; the judge does all timing.
 *
 * Solution convention: `public class Solution` defines a `public static` method with the
 * name in signature.json. Protocol types map to Java types as follows:
 * int -> long/Long, float -> double/Double, bool -> boolean/Boolean, string -> String,
 * list -> a Java array of the element's mapped type, tuple -> Object[], null -> Object.
 * A `nullable` descriptor needs a boxed/reference parameter so null can pass through.
 */
public final class Driver {

    static final int PROTOCOL_VERSION = 1;
    static final String SIGNATURE_PATH = "/box/signature.json";
    static final int EXIT_PROTOCOL = 2;
    static final int EXIT_USER = 1;
    static final String CALL_STDERR_MARKER = "\u001ecounterspell:stderr\n";
    /** The largest integer every launch driver can hold exactly (2^53 - 1, as Python and JS). */
    static final long MAX_SAFE_INTEGER = (1L << 53) - 1;
    static final int STDERR_CAP = 64 * 1024;
    static final String STDERR_TRUNCATED = "\n[stderr truncated]\n";

    private Driver() {}

    /** A setup or protocol failure. The driver reports it on stderr and exits non-zero. */
    static final class DriverFailure extends Exception {
        final int exitCode;
        DriverFailure(String message) { this(message, EXIT_PROTOCOL); }
        DriverFailure(String message, int exitCode) {
            super(message);
            this.exitCode = exitCode;
        }
    }

    /** A parsed type descriptor from signature.json. */
    static final class TypeDescriptor {
        final String kind;
        final boolean nullable;
        final List<TypeDescriptor> items;
        TypeDescriptor(String kind, boolean nullable, List<TypeDescriptor> items) {
            this.kind = kind;
            this.nullable = nullable;
            this.items = items;
        }
    }

    // ------------------------------------------------------------------
    // Minimal JSON reader/writer: protocol v1 needs only JSON-safe values.
    // ------------------------------------------------------------------

    static final class Json {
        private final String text;
        private int pos;

        private Json(String text) { this.text = text; }

        static Object parse(String text) throws DriverFailure {
            Json reader = new Json(text);
            Object value = reader.readValue();
            reader.skipSpace();
            if (reader.pos != reader.text.length()) {
                throw new DriverFailure("trailing data after the JSON value");
            }
            return value;
        }

        private void skipSpace() {
            while (pos < text.length() && " \t\n\r".indexOf(text.charAt(pos)) >= 0) pos++;
        }

        private char peek() throws DriverFailure {
            if (pos >= text.length()) throw new DriverFailure("unexpected end of JSON");
            return text.charAt(pos);
        }

        private void expect(char c) throws DriverFailure {
            if (peek() != c) throw new DriverFailure("expected '" + c + "' in JSON");
            pos++;
        }

        private Object readValue() throws DriverFailure {
            skipSpace();
            char c = peek();
            switch (c) {
                case '{': return readObject();
                case '[': return readArray();
                case '"': return readString();
                case 't': return readLiteral("true", Boolean.TRUE);
                case 'f': return readLiteral("false", Boolean.FALSE);
                case 'n': return readLiteral("null", null);
                default: return readNumber();
            }
        }

        private Object readLiteral(String word, Object value) throws DriverFailure {
            if (!text.startsWith(word, pos)) throw new DriverFailure("bad JSON literal");
            pos += word.length();
            return value;
        }

        private Map<String, Object> readObject() throws DriverFailure {
            expect('{');
            Map<String, Object> out = new LinkedHashMap<>();
            skipSpace();
            if (peek() == '}') { pos++; return out; }
            while (true) {
                skipSpace();
                String key = readString();
                skipSpace();
                expect(':');
                out.put(key, readValue());
                skipSpace();
                char c = peek();
                if (c == '}') { pos++; return out; }
                expect(',');
            }
        }

        private List<Object> readArray() throws DriverFailure {
            expect('[');
            List<Object> out = new ArrayList<>();
            skipSpace();
            if (peek() == ']') { pos++; return out; }
            while (true) {
                out.add(readValue());
                skipSpace();
                char c = peek();
                if (c == ']') { pos++; return out; }
                expect(',');
            }
        }

        private String readString() throws DriverFailure {
            expect('"');
            StringBuilder sb = new StringBuilder();
            while (true) {
                char c = peek();
                pos++;
                switch (c) {
                    case '"':
                        return sb.toString();
                    case '\\':
                        char esc = peek();
                        pos++;
                        switch (esc) {
                            case '"': sb.append('"'); break;
                            case '\\': sb.append('\\'); break;
                            case '/': sb.append('/'); break;
                            case 'b': sb.append('\b'); break;
                            case 'f': sb.append('\f'); break;
                            case 'n': sb.append('\n'); break;
                            case 'r': sb.append('\r'); break;
                            case 't': sb.append('\t'); break;
                            case 'u':
                                if (pos + 4 > text.length()) {
                                    throw new DriverFailure("truncated \\u escape");
                                }
                                int code;
                                try {
                                    code = Integer.parseInt(text.substring(pos, pos + 4), 16);
                                } catch (NumberFormatException e) {
                                    throw new DriverFailure("bad \\u escape");
                                }
                                sb.append((char) code);
                                pos += 4;
                                break;
                            default:
                                throw new DriverFailure("bad string escape \\" + esc);
                        }
                        break;
                    default:
                        sb.append(c);
                }
            }
        }

        private Object readNumber() throws DriverFailure {
            int start = pos;
            if (peek() == '-') pos++;
            while (pos < text.length() && Character.isDigit(text.charAt(pos))) pos++;
            boolean isFloat = false;
            if (pos < text.length() && text.charAt(pos) == '.') {
                isFloat = true;
                pos++;
                while (pos < text.length() && Character.isDigit(text.charAt(pos))) pos++;
            }
            if (pos < text.length() && (text.charAt(pos) == 'e' || text.charAt(pos) == 'E')) {
                isFloat = true;
                pos++;
                if (pos < text.length() && (text.charAt(pos) == '+' || text.charAt(pos) == '-')) pos++;
                while (pos < text.length() && Character.isDigit(text.charAt(pos))) pos++;
            }
            if (pos == start || (pos == start + 1 && text.charAt(start) == '-')) {
                throw new DriverFailure("bad JSON number");
            }
            String digits = text.substring(start, pos);
            try {
                return isFloat ? (Object) Double.valueOf(digits) : Long.valueOf(digits);
            } catch (NumberFormatException e) {
                // Integer literals past long range arrive as a double only when the
                // judge generates them; drivers never coerce, so refuse the line.
                try {
                    double d = Double.parseDouble(digits);
                    if (Double.isFinite(d)) return d;
                } catch (NumberFormatException ignored) {
                    // fall through
                }
                throw new DriverFailure("bad JSON number " + digits);
            }
        }

        static String write(Object value) throws DriverFailure {
            StringBuilder sb = new StringBuilder();
            writeValue(sb, value);
            return sb.toString();
        }

        private static void writeValue(StringBuilder sb, Object value) throws DriverFailure {
            if (value == null) {
                sb.append("null");
            } else if (value instanceof Boolean) {
                sb.append(value);
            } else if (value instanceof Long || value instanceof Integer || value instanceof Short
                    || value instanceof Byte) {
                sb.append(((Number) value).longValue());
            } else if (value instanceof Double || value instanceof Float) {
                double d = ((Number) value).doubleValue();
                if (!Double.isFinite(d)) {
                    throw new DriverFailure("the return value has a non-finite float");
                }
                sb.append(d);
            } else if (value instanceof String) {
                writeString(sb, (String) value);
            } else if (value instanceof List) {
                sb.append('[');
                boolean first = true;
                for (Object item : (List<?>) value) {
                    if (!first) sb.append(',');
                    first = false;
                    writeValue(sb, item);
                }
                sb.append(']');
            } else if (value.getClass().isArray()) {
                sb.append('[');
                int length = Array.getLength(value);
                for (int i = 0; i < length; i++) {
                    if (i > 0) sb.append(',');
                    writeValue(sb, Array.get(value, i));
                }
                sb.append(']');
            } else {
                throw new DriverFailure("the return value is not JSON-safe: " + value.getClass().getName());
            }
        }

        private static void writeString(StringBuilder sb, String value) {
            sb.append('"');
            for (int i = 0; i < value.length(); i++) {
                char c = value.charAt(i);
                switch (c) {
                    case '"': sb.append("\\\""); break;
                    case '\\': sb.append("\\\\"); break;
                    case '\b': sb.append("\\b"); break;
                    case '\f': sb.append("\\f"); break;
                    case '\n': sb.append("\\n"); break;
                    case '\r': sb.append("\\r"); break;
                    case '\t': sb.append("\\t"); break;
                    default:
                        if (c < 0x20) {
                            sb.append(String.format("\\u%04x", (int) c));
                        } else {
                            sb.append(c);
                        }
                }
            }
            sb.append('"');
        }
    }

    // ------------------------------------------------------------------
    // Signature descriptor.
    // ------------------------------------------------------------------

    static TypeDescriptor checkType(Object raw) throws DriverFailure {
        if (!(raw instanceof Map)) {
            throw new DriverFailure("a type in the signature is not an object");
        }
        Map<?, ?> descriptor = (Map<?, ?>) raw;
        Object kind = descriptor.get("kind");
        if (!(kind instanceof String)
                || !List.of("int", "float", "bool", "string", "list", "tuple", "null")
                        .contains(kind)) {
            throw new DriverFailure("unknown type kind " + kind + " in the signature");
        }
        Object nullable = descriptor.get("nullable");
        if (nullable != null && !(nullable instanceof Boolean)) {
            throw new DriverFailure("the nullable flag must be true or false");
        }
        List<TypeDescriptor> items = List.of();
        if (kind.equals("list")) {
            items = List.of(checkType(descriptor.get("items")));
        } else if (kind.equals("tuple")) {
            Object rawItems = descriptor.get("items");
            if (!(rawItems instanceof List)) {
                throw new DriverFailure("a tuple type needs a list of item types");
            }
            List<TypeDescriptor> parsed = new ArrayList<>();
            for (Object item : (List<?>) rawItems) parsed.add(checkType(item));
            items = List.copyOf(parsed);
        }
        return new TypeDescriptor((String) kind, Boolean.TRUE.equals(nullable), items);
    }

    static final class Signature {
        final String functionName;
        final List<TypeDescriptor> parameters;
        final TypeDescriptor returns;
        Signature(String functionName, List<TypeDescriptor> parameters, TypeDescriptor returns) {
            this.functionName = functionName;
            this.parameters = parameters;
            this.returns = returns;
        }
    }

    static Signature loadSignature() throws DriverFailure {
        String path = System.getenv().getOrDefault("COUNTER_SPELL_SIGNATURE", SIGNATURE_PATH);
        Object data;
        try {
            data = Json.parse(Files.readString(Path.of(path), StandardCharsets.UTF_8));
        } catch (IOException e) {
            throw new DriverFailure("cannot read the signature: " + e.getMessage());
        }
        if (!(data instanceof Map)) {
            throw new DriverFailure("the signature is not an object");
        }
        Map<?, ?> raw = (Map<?, ?>) data;
        Object version = raw.get("protocol_version");
        if (!(version instanceof Number) || ((Number) version).longValue() != PROTOCOL_VERSION) {
            throw new DriverFailure("the signature has an unsupported protocol version");
        }
        Object function = raw.get("function");
        if (!(function instanceof String)
                || !((String) function).matches("[A-Za-z_$][A-Za-z0-9_$]*")) {
            throw new DriverFailure("the signature has no valid function name");
        }
        Object parameters = raw.get("parameters");
        if (!(parameters instanceof List)) {
            throw new DriverFailure("the signature has no parameter list");
        }
        List<TypeDescriptor> parsedParams = new ArrayList<>();
        for (Object param : (List<?>) parameters) {
            if (!(param instanceof Map)) {
                throw new DriverFailure("a parameter in the signature is not an object");
            }
            parsedParams.add(checkType(((Map<?, ?>) param).get("type")));
        }
        return new Signature((String) function, List.copyOf(parsedParams), checkType(raw.get("returns")));
    }

    // ------------------------------------------------------------------
    // Value checking and coercion.
    // ------------------------------------------------------------------

    /** True when a parsed JSON value matches a type descriptor. Never converts the value. */
    static boolean conforms(Object value, TypeDescriptor descriptor) {
        if (value == null) {
            return descriptor.kind.equals("null") || descriptor.nullable;
        }
        switch (descriptor.kind) {
            case "int":
                return value instanceof Long
                        && (Long) value >= -MAX_SAFE_INTEGER
                        && (Long) value <= MAX_SAFE_INTEGER;
            case "float":
                return value instanceof Double && Double.isFinite((Double) value);
            case "bool":
                return value instanceof Boolean;
            case "string":
                return value instanceof String;
            case "list":
                if (!(value instanceof List)) return false;
                for (Object item : (List<?>) value) {
                    if (!conforms(item, descriptor.items.get(0))) return false;
                }
                return true;
            case "tuple":
                if (!(value instanceof List)) return false;
                List<?> items = (List<?>) value;
                if (items.size() != descriptor.items.size()) return false;
                for (int i = 0; i < items.size(); i++) {
                    if (!conforms(items.get(i), descriptor.items.get(i))) return false;
                }
                return true;
            default:
                return false;
        }
    }

    /** The Java type a descriptor maps to, for method resolution. */
    static Class<?> javaType(TypeDescriptor descriptor) throws DriverFailure {
        switch (descriptor.kind) {
            case "int": return descriptor.nullable ? Long.class : long.class;
            case "float": return descriptor.nullable ? Double.class : double.class;
            case "bool": return descriptor.nullable ? Boolean.class : boolean.class;
            case "string": return String.class;
            case "list":
                return Array.newInstance(javaType(descriptor.items.get(0)), 0).getClass();
            case "tuple":
                return Object[].class;
            default:
                return Object.class; // kind "null" only ever carries null
        }
    }

    /** Converts a checked JSON value into the Java value a solution parameter expects. */
    static Object coerce(Object value, TypeDescriptor descriptor) throws DriverFailure {
        if (value == null) return null;
        switch (descriptor.kind) {
            case "list":
                List<?> items = (List<?>) value;
                Class<?> component = javaType(descriptor.items.get(0));
                Object array = Array.newInstance(component, items.size());
                for (int i = 0; i < items.size(); i++) {
                    Array.set(array, i, coerce(items.get(i), descriptor.items.get(0)));
                }
                return array;
            case "tuple":
                List<?> parts = (List<?>) value;
                Object[] out = new Object[parts.size()];
                for (int i = 0; i < parts.size(); i++) {
                    Object part = coerce(parts.get(i), descriptor.items.get(i));
                    if (part != null && part.getClass().isPrimitive()) {
                        throw new DriverFailure("unreachable: primitive in Object[]");
                    }
                    out[i] = part;
                }
                return out;
            default:
                return value;
        }
    }

    /** Boxes a returned Java value into the JSON value space for the conformance check. */
    static Object normalize(Object value) throws DriverFailure {
        if (value == null) return null;
        Class<?> type = value.getClass();
        if (value instanceof Integer || value instanceof Short || value instanceof Byte) {
            return ((Number) value).longValue();
        }
        if (value instanceof Float) {
            return ((Number) value).doubleValue();
        }
        if (type.isArray()) {
            int length = Array.getLength(value);
            List<Object> out = new ArrayList<>(length);
            for (int i = 0; i < length; i++) out.add(normalize(Array.get(value, i)));
            return out;
        }
        if (value instanceof List) {
            List<Object> out = new ArrayList<>(((List<?>) value).size());
            for (Object item : (List<?>) value) out.add(normalize(item));
            return out;
        }
        if (value instanceof Long || value instanceof Double || value instanceof Boolean
                || value instanceof String) {
            return value;
        }
        throw new DriverFailure("the return value is not JSON-safe: " + type.getName());
    }

    // ------------------------------------------------------------------
    // Solution loading and calls.
    // ------------------------------------------------------------------

    /** Finds the public static Solution method matching the signature name and Java types. */
    static Method resolveFunction(Signature signature) throws DriverFailure {
        Class<?> solution;
        try {
            solution = Class.forName("Solution");
        } catch (ClassNotFoundException e) {
            throw new DriverFailure("cannot load the Solution class");
        }
        List<Class<?>> wanted = new ArrayList<>();
        for (TypeDescriptor parameter : signature.parameters) {
            wanted.add(javaType(parameter));
        }
        for (Method method : solution.getDeclaredMethods()) {
            if (!Modifier.isStatic(method.getModifiers())
                    || !Modifier.isPublic(method.getModifiers())
                    || !method.getName().equals(signature.functionName)
                    || method.getParameterCount() != wanted.size()) {
                continue;
            }
            Class<?>[] actual = method.getParameterTypes();
            boolean matches = true;
            for (int i = 0; i < actual.length; i++) {
                if (!actual[i].equals(wanted.get(i))) {
                    matches = false;
                    break;
                }
            }
            if (matches) return method;
        }
        throw new DriverFailure("the solution does not define " + signature.functionName
                + " with the signature's Java types");
    }

    /** A tee that keeps the sandbox's first-64KiB stderr copy and mirrors live bytes. */
    static final class BoundedErr extends ByteArrayOutputStream {
        private final PrintStream mirror;
        private boolean truncated;
        BoundedErr(PrintStream mirror) {
            this.mirror = mirror;
        }
        @Override public synchronized void write(int b) {
            mirror.write(b);
            if (size() < STDERR_CAP) {
                super.write(b);
            } else {
                truncated = true;
            }
        }
        @Override public synchronized void write(byte[] bytes, int offset, int length) {
            mirror.write(bytes, offset, length);
            int take = Math.min(length, STDERR_CAP - size());
            if (take > 0) {
                super.write(bytes, offset, take);
            }
            truncated |= take < length;
        }
        String text() {
            return toString(StandardCharsets.UTF_8) + (truncated ? STDERR_TRUNCATED : "");
        }
    }

    /** Hides stdout and stdin and captures stderr while user code runs. */
    static final class Quiet implements AutoCloseable {
        private final PrintStream savedOut;
        private final java.io.InputStream savedIn;
        private final PrintStream savedErr;
        private final BoundedErr capturedErr;
        Quiet() {
            savedOut = System.out;
            savedIn = System.in;
            savedErr = System.err;
            capturedErr = new BoundedErr(savedErr);
            System.setOut(new PrintStream(new ByteArrayOutputStream(), true, StandardCharsets.UTF_8));
            System.setIn(new ByteArrayInputStream(new byte[0]));
            System.setErr(new PrintStream(capturedErr, true, StandardCharsets.UTF_8));
        }
        String stderr() {
            return capturedErr.text();
        }
        @Override public void close() {
            System.setOut(savedOut);
            System.setIn(savedIn);
            System.setErr(savedErr);
        }
    }

    static void diagnose(String message) {
        System.err.println("driver: " + message);
    }

    // ------------------------------------------------------------------
    // Call loop.
    // ------------------------------------------------------------------

    /** A validated call: its id plus the arguments coerced to Java values. */
    static final class Call {
        final long id;
        final Object[] args;
        Call(long id, Object[] args) {
            this.id = id;
            this.args = args;
        }
    }

    static Call parseCall(String line, Signature signature) throws DriverFailure {
        Object data = Json.parse(line);
        if (!(data instanceof Map)) {
            throw new DriverFailure("a call line is not a JSON object");
        }
        Map<?, ?> call = (Map<?, ?>) data;
        Object t = call.get("t");
        if (!(t instanceof Number) || ((Number) t).longValue() < 0) {
            throw new DriverFailure("a call line has no valid call id");
        }
        Object args = call.get("args");
        if (!(args instanceof List)) {
            throw new DriverFailure("a call line has no argument list");
        }
        List<?> argList = (List<?>) args;
        if (argList.size() != signature.parameters.size()) {
            throw new DriverFailure("a call line has the wrong argument count");
        }
        Object[] javaArgs = new Object[argList.size()];
        for (int i = 0; i < argList.size(); i++) {
            TypeDescriptor descriptor = signature.parameters.get(i);
            if (!conforms(argList.get(i), descriptor)) {
                throw new DriverFailure("an argument does not match the signature");
            }
            javaArgs[i] = coerce(argList.get(i), descriptor);
        }
        return new Call(((Number) t).longValue(), javaArgs);
    }

    public static void main(String[] args) {
        String mode = "normal";
        for (int i = 0; i < args.length; i++) {
            if (args[i].equals("--mode") && i + 1 < args.length) {
                mode = args[i + 1];
                i++;
            }
        }
        if (!mode.equals("normal") && !mode.equals("bench")) {
            diagnose("unknown mode " + mode);
            System.exit(EXIT_PROTOCOL);
        }
        try {
            Signature signature = loadSignature();
            Method function = resolveFunction(signature);
            serve(function, signature);
        } catch (DriverFailure failure) {
            diagnose(failure.getMessage());
            System.exit(failure.exitCode);
        }
    }

    static void serve(Method function, Signature signature) throws DriverFailure {
        BufferedReader requests = new BufferedReader(
                new InputStreamReader(System.in, StandardCharsets.UTF_8));
        PrintWriter answers = new PrintWriter(
                new OutputStreamWriter(System.out, StandardCharsets.UTF_8));
        String line;
        try {
            while ((line = requests.readLine()) != null) {
                // Marks this call's process-stderr boundary. Successful user stderr travels
                // in the answer instead, so delayed process output can be discarded safely.
                System.err.print(CALL_STDERR_MARKER);
                if (line.isBlank()) {
                    throw new DriverFailure("a blank call line is not valid protocol");
                }
                Call call = parseCall(line, signature);
                Object result;
                String stderr;
                Quiet quiet = new Quiet();
                try {
                    result = function.invoke(null, call.args);
                } catch (InvocationTargetException e) {
                    stderr = quiet.stderr();
                    quiet.close();
                    diagnose("the solution raised an exception");
                    e.getCause().printStackTrace();
                    System.exit(EXIT_USER);
                    return;
                } catch (IllegalArgumentException | IllegalAccessException e) {
                    stderr = quiet.stderr();
                    quiet.close();
                    diagnose("the solution failed while loading");
                    e.printStackTrace();
                    System.exit(EXIT_PROTOCOL);
                    return;
                } finally {
                    stderr = quiet.stderr();
                    quiet.close();
                }
                Object normalized;
                try {
                    normalized = normalize(result);
                } catch (DriverFailure failure) {
                    throw failure;
                }
                if (!conforms(normalized, signature.returns)) {
                    throw new DriverFailure("the return value is not JSON-safe for the signature");
                }
                answers.print("{\"t\":" + call.id + ",\"out\":" + Json.write(normalized));
                if (!stderr.isEmpty()) {
                    answers.print(",\"stderr\":" + Json.write(stderr));
                }
                answers.println("}");
                answers.flush();
            }
        } catch (IOException e) {
            throw new DriverFailure("the call stream failed: " + e.getMessage());
        }
    }
}

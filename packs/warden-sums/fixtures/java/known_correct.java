// Test-only known-correct Java fixture for warden-sums (language-drivers T13).
// Convention: a public static method on `Solution` whose parameter and return
// types are the driver's mapped Java types (list[int] -> long[], int -> long).
public class Solution {
    public static long ritual_spans(long[] readings, long quota) {
        java.util.Map<Long, Long> seen = new java.util.HashMap<>();
        seen.put(0L, 1L);
        long total = 0;
        long spans = 0;
        for (long reading : readings) {
            total += reading;
            spans += seen.getOrDefault(total - quota, 0L);
            seen.merge(total, 1L, Long::sum);
        }
        return spans;
    }
}

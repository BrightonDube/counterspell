// Test-only known-correct Java fixture for harvest-tally (language-drivers T13).
// Convention: a public static method on `Solution` whose parameter and return
// types are the driver's mapped Java types (list[int] -> long[], int -> long).
// The tally outgrows int, so every accumulator is a long.
public class Solution {
    public static long cross_pairs(long[] rows) {
        long total = 0;
        long ahead = 0;
        for (long count : rows) ahead += count;
        for (long count : rows) {
            ahead -= count;
            total += count * ahead;
        }
        return total;
    }
}

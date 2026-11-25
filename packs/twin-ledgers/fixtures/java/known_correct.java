// Test-only known-correct Java fixture for twin-ledgers (language-drivers T13).
// Convention: a public static method on `Solution` whose parameter and return
// types are the driver's mapped Java types (list[list[int]] -> long[][],
// int -> long).
public class Solution {
    public static long matching_totals(long[][] entries_a, long[][] entries_b) {
        long[] a = new long[51];
        long[] b = new long[51];
        for (long[] row : entries_a) a[(int) row[0]] += row[1];
        for (long[] row : entries_b) b[(int) row[0]] += row[1];
        java.util.Set<Long> seen_a = new java.util.HashSet<>();
        for (long[] row : entries_a) seen_a.add(row[0]);
        java.util.Set<Long> seen_b = new java.util.HashSet<>();
        for (long[] row : entries_b) seen_b.add(row[0]);
        long agreed = 0;
        for (long key : seen_a) {
            if (seen_b.contains(key) && a[(int) key] == b[(int) key]) agreed += 1;
        }
        return agreed;
    }
}

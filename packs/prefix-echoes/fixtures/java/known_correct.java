// Test-only known-correct Java fixture for prefix-echoes (language-drivers T13).
// Convention: a public static method on `Solution` whose parameter and return
// types are the driver's mapped Java types (list[list[int]] -> long[][],
// list[int] -> long[]).
public class Solution {
    public static long[] prefix_counts(long[] marks, long[][] queries) {
        long[] answers = new long[queries.length];
        for (int q = 0; q < queries.length; q += 1) {
            long count = 0;
            long k = Math.min(queries[q][0], marks.length);
            for (int i = 0; i < k; i += 1) {
                if (marks[i] == queries[q][1]) count += 1;
            }
            answers[q] = count;
        }
        return answers;
    }
}

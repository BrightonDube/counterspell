// Test-only known-correct Java fixture for count-pairs (language-drivers T13).
// Convention: a public static method on `Solution` whose parameter and return
// types are the driver's mapped Java types (list[int] -> long[], int -> long).
public class Solution {
    public static long count_pairs(long[] values, long target) {
        long pairs = 0;
        for (int left = 0; left < values.length; left += 1) {
            for (int right = left + 1; right < values.length; right += 1) {
                if (values[left] + values[right] == target) {
                    pairs += 1;
                }
            }
        }
        return pairs;
    }
}

// Test-only known-correct Java fixture for candle-frequency (language-drivers T13).
// Convention: a public static method on `Solution` whose parameter and return
// types are the driver's mapped Java types (list[int] -> long[], int -> long).
public class Solution {
    public static long top_burn(long[] sizes) {
        long[] counts = new long[10];
        for (long size : sizes) counts[(int) size] += 1;
        long best = 0;
        for (int size = 9; size >= 1; size -= 1) {
            if (counts[size] > 0 && counts[size] >= counts[(int) best]) best = size;
        }
        return best;
    }
}

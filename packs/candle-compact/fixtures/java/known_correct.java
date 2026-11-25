// Test-only known-correct Java fixture for candle-compact (language-drivers T13).
// Convention: a public static method on `Solution` whose parameter and return
// types are the driver's mapped Java types (list[int] -> long[], int -> long).
public class Solution {
    public static long remaining_candles(long[] marks, long cursed) {
        long kept = 0;
        for (long mark : marks) {
            if (mark != cursed) {
                kept += 1;
            }
        }
        return kept;
    }
}

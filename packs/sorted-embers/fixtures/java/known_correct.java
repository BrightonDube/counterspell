// Test-only known-correct Java fixture for sorted-embers (language-drivers T13).
// Convention: a public static method on `Solution` whose parameter and return
// types are the driver's mapped Java types (list[int] -> long[]).
public class Solution {
    public static long[] ordered_squares(long[] readings) {
        long[] answer = new long[readings.length];
        int left = 0, right = readings.length - 1;
        for (int write = readings.length - 1; write >= 0; write -= 1) {
            long lo = readings[left] * readings[left];
            long hi = readings[right] * readings[right];
            if (hi >= lo) {
                answer[write] = hi;
                right -= 1;
            } else {
                answer[write] = lo;
                left += 1;
            }
        }
        return answer;
    }
}

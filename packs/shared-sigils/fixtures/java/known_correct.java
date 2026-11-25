// Test-only known-correct Java fixture for shared-sigils (language-drivers T13).
// Convention: a public static method on `Solution` whose parameter and return
// types are the driver's mapped Java types (list[int] -> long[]).
public class Solution {
    public static long[] shared_marks(long[] first, long[] second) {
        long[] buffer = new long[Math.min(first.length, second.length)];
        int shared = 0;
        int a = 0, b = 0;
        while (a < first.length && b < second.length) {
            if (first[a] < second[b]) {
                a += 1;
            } else if (second[b] < first[a]) {
                b += 1;
            } else {
                long mark = first[a];
                if (shared == 0 || buffer[shared - 1] != mark) {
                    buffer[shared] = mark;
                    shared += 1;
                }
                a += 1;
                b += 1;
            }
        }
        long[] answer = new long[shared];
        for (int i = 0; i < shared; i += 1) answer[i] = buffer[i];
        return answer;
    }
}

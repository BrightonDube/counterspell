// Test-only known-correct Java fixture for equal-embers (language-drivers T13).
// Convention: a public static method on `Solution` whose parameter and return
// types are the driver's mapped Java types (list[int] -> long[], int -> long).
public class Solution {
    public static long balanced_span(long[] embers) {
        java.util.Map<Long, Integer> firstSeen = new java.util.HashMap<>();
        firstSeen.put(0L, -1);
        long balance = 0;
        long longest = 0;
        for (int index = 0; index < embers.length; index += 1) {
            balance += embers[index] == 1 ? 1 : -1;
            if (firstSeen.containsKey(balance)) {
                longest = Math.max(longest, index - firstSeen.get(balance));
            } else {
                firstSeen.put(balance, index);
            }
        }
        return longest;
    }
}

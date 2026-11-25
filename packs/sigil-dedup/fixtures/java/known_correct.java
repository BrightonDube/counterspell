// Test-only known-correct Java fixture for sigil-dedup (language-drivers T13).
// Convention: a public static method on `Solution` whose parameter and return
// types are the driver's mapped Java types (list[int] -> long[], int -> long).
public class Solution {
    public static long distinct_sigils(long[] marks) {
        long distinct = 0;
        for (int index = 0; index < marks.length; index += 1) {
            if (index == 0 || marks[index] != marks[index - 1]) {
                distinct += 1;
            }
        }
        return distinct;
    }
}

// Test-only known-correct Java fixture for twin-stems.
public class Solution {
    public static long prefix_pairs(String words) {
        java.util.Set<String> set = new java.util.TreeSet<>();
        for (String w : words.split(",", -1)) {
            if (!w.isEmpty()) {
                set.add(w);
            }
        }
        String[] ws = set.toArray(new String[0]);
        long ans = 0;
        for (int i = 0; i < ws.length; i += 1) {
            for (int j = i + 1; j < ws.length; j += 1) {
                if (!ws[i].equals(ws[j]) && ws[j].startsWith(ws[i])) {
                    ans += 1;
                }
            }
        }
        return ans;
    }
}

// KNOWN MISS - four real faults, one caught. This file is not expected to pass and it is
// not expected to be fully caught either: it is the receipt for a limit we know about.
//
// line 8  a promise dropped on the floor      MISSED - the fault is a missing `await`
// line 9  a string thrown instead of an Error MISSED - `Error` is erased by the NOISE list,
//                                                      which lowercases before comparing
// line 10 `||` where `??` is meant            MISSED - punctuation is a separator, never a
//                                                      token, so `??` and `||` cannot exist
// line 11 path segments joined with a slash   CAUGHT - `path.join` is a real word
//
// All three misses are ABSENCES, and the gate finds a fault by what the wrong line CARRIES.
// Two of them are destroyed by the tokeniser before any gate could see them, which is why
// the rules were struck on 24 Sep rather than rewritten - see README.md, the third class.
//
// THE DAY THE TOKENISER IS FIXED - case-sensitive NOISE, operators as tokens - this file
// goes from 1 hit to 4 and proves it. That is the whole point of keeping it.
export async function go(dir, name, req) {
  doWork();
  if (!name) throw 'tool not found';
  const limit = req.query.limit || 20;
  const file = dir + '/' + name + '.json';
  return { limit, file };
}

# sarge — for LangChain

Your LangGraph agent, with a drill sergeant in it. A small local model writes the code. The
rules of your repo ride at the tail of every turn. Every file it writes is checked against
them. The run refuses while a rule is broken. When it is stuck, a tutor answers and the answer
becomes a rule. The model and the check run on your machine. The tutor call is the one
thing that leaves: the error, the lines around it, and the files the model wrote. Never
the rest of your repo. No tutor key, nothing leaves.

```bash
pip install -e python        # from your Sarge clone. (Not on PyPI yet; when it is, the package is `sarge-rules` — `sarge` there is someone else's.)
```

```python
from sarge import Sarge

agent = Sarge(repo=".", task="Add GET /tools/stats ...").agent()
agent.invoke({"messages": [("user", "Add GET /tools/stats ...")]})
```

Keep your own graph and take the pieces:

```python
from langchain.agents import create_agent
s = Sarge(repo=".", task=task)
graph = create_agent(s.llm(), s.tools(), system_prompt=s.character(), middleware=[s.rules(), s.stop()])
```

| piece | what it is |
|---|---|
| `s.llm()` | the organ — llama.cpp with the Sarge handshake compiled in — as `ChatOpenAI` on a local port |
| `s.tools()` | `list_files` `read_file` `edit_file` `write_file` `run_build` `run_app` `ask_tutor`; the check after every write, the refusal in `run_app`, the forced ask, the four-strike stop |
| `s.rules()` | middleware: the framed `.sarge` block appended as the last message on every model call, from disk |
| `s.stop()` | middleware: the graph ends when the harness has cancelled the run |
| `s.character()` | `harness/CHARACTER.md` plus the working instructions, as the system prompt |

Needs a Sarge clone (`SARGE_HOME`, or install this package from inside one) with the
handshake built and the organ running. See the repo README for the from-scratch steps.

Reference host, the same command line as the .NET one:

```
python -m sarge --repo <dir> --task "<text>" --harness
```

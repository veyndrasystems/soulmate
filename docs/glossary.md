# Translate the record into your existing workflow

You can ask your host one question: **what still needs doing before I can
accept this change?** These terms explain the references behind its answer;
they are not extra steps to memorize.

| Soulmate term | Familiar equivalent | Distinction to keep |
| --- | --- | --- |
| Assignment / brief | A bounded subagent task | Reading it does not spawn an agent. |
| Worker submission | The worker's result document | A completion claim is not a passing test. |
| Check report | The command and exit the host reports | Soulmate does not execute or authenticate the test. |
| Reviewer finding | A separate review of the change | Approval still needs a lead decision. |
| Lead acceptance | The task owner's recorded decision | It neither merges Git changes nor proves the code correct. |
| Artifact | A referenced result document | Its recorded bytes must remain unchanged. Product files can change in later attempts. |
| Ledger / run | The task's inspectable sequence of records | It is not a recovered host conversation. |
| Rework | Another attempt within the frozen task | Use fresh result documents, checks, and review. |
| Supersede | An explicit successor after governing inputs change | The predecessor remains inspectable; final runs stay final. |
| Declared boundary | The task's stated file/command limits | The host owns execution permissions; this is not a sandbox. |
| ControlRoot | Configuration and profiles directory | Portable mode puts it in the project; local mode keeps it elsewhere. |
| ProductRoot | The project being changed | It is distinct from private records. |
| StateRoot | Private records directory | An ignore rule is not access control. |
| Receipt | A snapshot of selected configuration references | It does not prove actions were performed. |

[Do the next change](first-checked-run.md#use-your-own-project) ·
[Repair or resume](repair-a-run.md) ·
[Optional surfaces](optional-surfaces.md).
The [reference](../REFERENCE.md) remains canonical for flags and formats.

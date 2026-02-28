# Document Router — Thai Voice-to-Text

อ่านเฉพาะไฟล์ที่ match task (1-2 ไฟล์ ไม่เกิน 35K)
สำหรับ implementation → อ่าน `designs/` | สำหรับ rationale/benchmark → อ่าน `research/`

---

## Audio & Recording

| Task / Topic | Read This File | Section |
|---|---|---|
| Audio capture, microphone, cpal | `designs/audio-pipeline-design.md` | §2 Audio Capture |
| Resampling, sample rate, 16kHz | `designs/audio-pipeline-design.md` | §3 Resampling |
| Ring buffer, audio buffering | `designs/audio-pipeline-design.md` | §4 Ring Buffer |
| VAD, voice activity detection, silence | `designs/audio-pipeline-design.md` | §5 VAD |
| Audio quality, validation, clipping | `designs/audio-pipeline-design.md` | §6 Quality Validation |
| Waveform visualization | `designs/audio-pipeline-design.md` | §7 Waveform Data |
| Audio research, cpal alternatives | `research/research-thai-voice-to-text.md` | §3 Audio Capture |

## STT (Speech-to-Text)

| Task / Topic | Read This File | Section |
|---|---|---|
| Whisper integration, whisper-rs, GGML | `designs/stt-engine-design.md` | §2 WhisperEngine |
| Model management, download, GGML files | `designs/stt-engine-design.md` | §3 Model Management |
| STT performance, latency, GPU accel | `designs/stt-engine-design.md` | §4-5 Performance & GPU |
| Thai STT models, Thonburian Whisper | `research/research-thai-voice-to-text.md` | §1 Thai STT Models |
| STT comparison, ElevenLabs, Google | `research/comparison-thai-stt.md` | §1-2 STT Engines & Competitors |
| Rust Whisper bindings, whisper-rs vs candle | `research/research-thai-voice-to-text.md` | §2 Rust Integration |

## LLM Post-Processing

| Task / Topic | Read This File | Section |
|---|---|---|
| LLM pipeline, API provider, correction flow | `designs/llm-correction-design.md` | §1-2 Pipeline & Backend |
| Prompt engineering, Thai correction prompts | `designs/llm-correction-design.md` | §3 Prompt Engineering |
| Context manager, rolling window, history | `designs/llm-correction-design.md` | §4 Context Manager |
| Domain detection (medical/legal/tech) | `designs/llm-correction-design.md` | §5 Domain Detection |
| User vocabulary, custom words | `designs/llm-correction-design.md` | §6 User Vocabulary |
| LLM quality eval, CER metrics | `designs/llm-correction-design.md` | §7 Quality Evaluation |
| LLM fallback, error handling | `designs/llm-correction-design.md` | §8 Fallback Strategy |
| Thai STT error patterns (tone/homophone) | `research/llm-post-processing-research.md` | §1 ปัญหาหลัก |
| LLM correction research, HyPoradise | `research/llm-post-processing-research.md` | §2 งานวิจัยหลัก |
| Context window strategy research | `research/llm-post-processing-research.md` | §3,6 Context Window |
| Local LLM options (Qwen, Typhoon) | `research/llm-post-processing-research.md` | §4 Local LLM |
| LLM performance budget | `research/llm-post-processing-research.md` | §7 Performance Budget |
| GEC deep research, error taxonomy | `research/research-compilation.md` | §1-2 Error Patterns & LLM |

## UI Widget

| Task / Topic | Read This File | Section |
|---|---|---|
| egui widget, UI states, wireframes | `designs/ui-widget-design.md` | §2-3 States & Implementation |
| Color scheme, Thai typography | `designs/ui-widget-design.md` | §4 Color Scheme |
| Interaction, drag, resize, hotkey visual | `designs/ui-widget-design.md` | §5 Interaction Design |
| System tray | `designs/ui-widget-design.md` | §7 System Tray |
| UI framework research, egui vs iced | `research/research-thai-voice-to-text.md` | §4 UI Framework |

## Text Injection & Hotkey

| Task / Topic | Read This File | Section |
|---|---|---|
| Clipboard, paste, arboard, enigo | `designs/text-injection-design.md` | §1-2 Strategy & Implementation |
| Platform-specific injection issues | `designs/text-injection-design.md` | §3 Platform Considerations |
| Clipboard restore after paste | `designs/text-injection-design.md` | §4 Clipboard Restore |
| Thai text validation, Unicode | `designs/text-injection-design.md` | §5 Thai Validation |
| Global hotkey, push-to-talk, rdev | `research/research-thai-voice-to-text.md` | §5 Global Hotkey |
| Text injection research | `research/research-thai-voice-to-text.md` | §6 Text Injection |

## Threading & Architecture

| Task / Topic | Read This File | Section |
|---|---|---|
| Architecture overview, module boundaries | `designs/architecture-overview.md` | §1-3 Context & Modules |
| Data flow, complete pipeline | `designs/architecture-overview.md` | §4 Data Flow |
| Traits, key interfaces | `designs/architecture-overview.md` | §5 Interfaces |
| Pipeline state machine | `designs/architecture-overview.md` | §6 State Machine |
| Thread architecture, channels, mpsc | `designs/threading-and-data-flow.md` | §1-2 Threads & Channels |
| Shared state, Arc, Mutex | `designs/threading-and-data-flow.md` | §3 Shared State |
| Pipeline orchestrator | `designs/threading-and-data-flow.md` | §4 Orchestrator |
| Startup/shutdown sequence | `designs/threading-and-data-flow.md` | §6,8 Startup & Shutdown |
| Error propagation | `designs/threading-and-data-flow.md` | §7 Error Propagation |

## Configuration & Modes

| Task / Topic | Read This File | Section |
|---|---|---|
| Operating modes (Fast/Standard/Context) | `designs/configuration-and-modes.md` | §1 Operating Modes |
| Settings structure, AppConfig | `designs/configuration-and-modes.md` | §2 Settings Structure |
| Settings persistence, TOML, dirs crate | `designs/configuration-and-modes.md` | §3 Persistence |
| Data directory layout | `designs/configuration-and-modes.md` | §4 Data Directory |
| First-run experience, setup wizard | `designs/configuration-and-modes.md` | §5 First-Run |
| Model selection UI | `designs/configuration-and-modes.md` | §6 Model Selection |
| System requirements check | `designs/configuration-and-modes.md` | §7 Requirements Check |

## Research & References

| Task / Topic | Read This File | Section |
|---|---|---|
| System requirements, RAM/CPU/GPU | `research/system-requirements.md` | §1-5 Tiers & Details |
| OS-specific requirements | `research/system-requirements.md` | §6 Requirements by OS |
| Cloud deployment options | `research/system-requirements.md` | §7 Cloud |
| Build requirements (developer) | `research/system-requirements.md` | §8 Build Requirements |
| Implementation code examples (Rust) | `research/implementation-guide.md` | §1-2 Architecture & Patterns |
| Performance optimization | `research/implementation-guide.md` | §3 Performance |
| Testing & validation | `research/implementation-guide.md` | §4 Testing |
| Academic papers, citations | `research/papers-and-references.md` | §1-2 Papers & Metrics |
| GitHub repos, HuggingFace models | `research/papers-and-references.md` | §6-7 Repos & Models |
| Competitor analysis | `research/comparison-thai-stt.md` | §2-3 Competitors & Recommendations |
| Phased delivery, roadmap | `designs/architecture-overview.md` | §9 Phased Delivery |

## Git Workflow

| Task / Topic | Read This File | Section |
|---|---|---|
| Branch naming, branch types | `git-workflow/branching-strategy.md` | §2 Branch Types & Naming |
| Branch lifetime rules | `git-workflow/branching-strategy.md` | §3 Lifetime Rules |
| Hotfix flow, phase branch flow | `git-workflow/branching-strategy.md` | §5 Workflow Diagrams |
| Branch name examples per phase | `git-workflow/branching-strategy.md` | §6 Naming Examples |
| Multi-agent module isolation (which agent owns which files) | `git-workflow/branching-strategy.md` | §7 Multi-Agent Parallel |
| Commit format, conventional commits | `git-workflow/commit-conventions.md` | §1-2 Format & Types |
| Commit scopes (audio/stt/llm/pipeline/...) | `git-workflow/commit-conventions.md` | §3 Scopes |
| Commit examples Phase 1-4 | `git-workflow/commit-conventions.md` | §5 Real Examples |
| commit-msg git hook validator | `git-workflow/commit-conventions.md` | §7 Git Hook |
| PR title, PR description template | `git-workflow/pr-workflow.md` | §2-3 Title & Template |
| PR author checklist (pre-open) | `git-workflow/pr-workflow.md` | §4 Author Checklist |
| PR review checklist | `git-workflow/pr-workflow.md` | §5 Review Checklist |
| PR size guidelines (XS/S/M/L/XL) | `git-workflow/pr-workflow.md` | §6 Size Guidelines |
| GitHub Actions CI YAML | `git-workflow/pr-workflow.md` | §8 GitHub Actions CI |
| Merge strategy (squash vs merge commit) | `git-workflow/pr-workflow.md` | §9 Merge Strategy |
| SemVer versioning scheme | `git-workflow/release-process.md` | §1 Versioning Scheme |
| Version → Phase map (v0.1.0 → v1.0.0) | `git-workflow/release-process.md` | §2 Version Phase Map |
| Release branch steps | `git-workflow/release-process.md` | §3 Release Workflow |
| CHANGELOG format | `git-workflow/release-process.md` | §4 CHANGELOG Format |
| Git tag commands | `git-workflow/release-process.md` | §5 Tag Commands |
| Hotfix release process | `git-workflow/release-process.md` | §6 Hotfix Release |
| Pre-release checklist | `git-workflow/release-process.md` | §7 Pre-Release Checklist |
| Platform artifacts (MSI/DMG/AppImage) | `git-workflow/release-process.md` | §8 Platform Artifacts |
| Multi-agent module isolation map | `git-workflow/multi-agent-workflow.md` | §2 Module Isolation |
| Parallel session protocol (orchestrator steps) | `git-workflow/multi-agent-workflow.md` | §3 Protocol |
| Dependency merge order (config→pipeline→audio→...) | `git-workflow/multi-agent-workflow.md` | §4 Dependency Order |
| Conflict resolution (Cargo.toml, main.rs) | `git-workflow/multi-agent-workflow.md` | §5 Conflict Resolution |
| Agent briefing template (subagent prompt) | `git-workflow/multi-agent-workflow.md` | §8 Agent Briefing |
| Claude worktree setup, `claude --worktree` | `git-workflow/claude-worktree-workflow.md` | §2 Manual Worktrees |
| Agent Teams setup, experimental flag | `git-workflow/claude-worktree-workflow.md` | §3 Agent Teams |
| Agent definition files (.claude/agents/) | `git-workflow/claude-worktree-workflow.md` | §4 Agent Definitions |
| Phase sprint plan, which agents run in parallel | `git-workflow/claude-worktree-workflow.md` | §5 Phase Sprint Plan |
| Worktree merge flow, squash merge order | `git-workflow/claude-worktree-workflow.md` | §7 Merge Flow |
| Step-by-step run instructions, copy-paste prompts | `git-workflow/run-agent-team.md` | §2-9 Steps |
| Merge commands per module | `git-workflow/run-agent-team.md` | §3,5,7,9 Merge Steps |
| Troubleshooting agent team issues | `git-workflow/run-agent-team.md` | §Troubleshooting |

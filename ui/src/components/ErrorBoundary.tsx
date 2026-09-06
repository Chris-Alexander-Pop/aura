import { Component, type ErrorInfo, type ReactNode } from "react"
import { reportFrontendCrash } from "@/lib/crash-report"

interface Props {
  children: ReactNode
}

interface State {
  error: Error | null
}

/** Catches render errors so a single panel failure doesn't blank the whole WebView. */
export default class ErrorBoundary extends Component<Props, State> {
  state: State = { error: null }

  static getDerivedStateFromError(error: Error): State {
    return { error }
  }

  componentDidCatch(error: Error, info: ErrorInfo): void {
    reportFrontendCrash({
      message: error.message || "React render error",
      stack: [error.stack, info.componentStack].filter(Boolean).join("\n"),
      kind: "uncaught",
    })
  }

  render() {
    if (this.state.error) {
      return (
        <div className="flex h-full flex-col items-center justify-center gap-3 bg-base px-6 text-center text-sm text-subtext1">
          <p className="text-text font-medium">This panel crashed</p>
          <p className="max-w-md text-xs text-overlay1 break-words">
            {this.state.error.message}
          </p>
          <button
            type="button"
            className="rounded-md bg-surface0 px-3 py-1.5 text-xs text-text hover:bg-surface1"
            onClick={() => this.setState({ error: null })}
          >
            Retry
          </button>
        </div>
      )
    }
    return this.props.children
  }
}

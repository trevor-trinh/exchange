/**
 * SDK error types
 */

export class SdkError extends Error {
  constructor(
    message: string,
    public readonly cause?: unknown
  ) {
    super(message);
    this.name = "SdkError";
  }
}

export class ApiError extends SdkError {
  constructor(
    message: string,
    public readonly status: number,
    public readonly code?: string,
    cause?: unknown
  ) {
    super(message, cause);
    this.name = "ApiError";
  }
}

export class WebSocketError extends SdkError {
  constructor(message: string, cause?: unknown) {
    super(message, cause);
    this.name = "WebSocketError";
  }
}

export class ValidationError extends SdkError {
  constructor(message: string, cause?: unknown) {
    super(message, cause);
    this.name = "ValidationError";
  }
}

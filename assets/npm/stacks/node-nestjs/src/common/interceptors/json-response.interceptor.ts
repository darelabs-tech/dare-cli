import { CallHandler, ExecutionContext, Injectable, NestInterceptor } from '@nestjs/common';
import { Observable } from 'rxjs';

/**
 * No-op interceptor placeholder. Reserved for cross-cutting JSON envelope work
 * (correlation ids, timing, etc.) — currently passthrough.
 */
@Injectable()
export class JsonResponseInterceptor implements NestInterceptor {
  intercept(_ctx: ExecutionContext, next: CallHandler): Observable<unknown> {
    return next.handle();
  }
}

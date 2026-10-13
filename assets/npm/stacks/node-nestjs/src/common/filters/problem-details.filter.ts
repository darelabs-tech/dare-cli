import {
  ArgumentsHost,
  Catch,
  ExceptionFilter,
  HttpException,
  HttpStatus,
} from '@nestjs/common';
import type { Request, Response } from 'express';

/**
 * RFC 7807 Problem Details — uniform error shape across the API.
 */
@Catch()
export class ProblemDetailsFilter implements ExceptionFilter {
  catch(exception: unknown, host: ArgumentsHost): void {
    const ctx = host.switchToHttp();
    const res = ctx.getResponse<Response>();
    const req = ctx.getRequest<Request>();

    const status =
      exception instanceof HttpException
        ? exception.getStatus()
        : HttpStatus.INTERNAL_SERVER_ERROR;

    const body =
      exception instanceof HttpException
        ? (exception.getResponse() as Record<string, unknown>)
        : { message: 'Internal server error' };

    res.status(status).json({
      type: `urn:problem:${status}`,
      title: typeof body.error === 'string' ? body.error : body.message,
      status,
      detail: Array.isArray(body.message) ? body.message.join('; ') : body.message,
      instance: req.url,
    });
  }
}

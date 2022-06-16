export interface ErrorPayload<T> {
    error: true;
    code: number;
    message: string;
    data: T;
}

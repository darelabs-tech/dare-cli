<?php

namespace App\Http\Requests;

use Illuminate\Foundation\Http\FormRequest;

class LoginRequest extends FormRequest
{
    public function authorize(): bool
    {
        return true;
    }

    /**
     * @return array<string, mixed>
     */
    public function rules(): array
    {
        return [
            'email' => ['required', 'email', 'max:254'],
            'password' => ['required', 'string', 'min:8', 'regex:/[A-Z]/', 'regex:/[0-9]/'],
        ];
    }

    /**
     * @return array<string, string>
     */
    public function messages(): array
    {
        return [
            'password.regex' => 'Password must contain an uppercase letter and a digit.',
        ];
    }
}

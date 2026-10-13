<?php

namespace App\Services;

use App\Repositories\UsersRepository;
use Illuminate\Auth\AuthenticationException;
use Illuminate\Support\Facades\Hash;

class AuthService
{
    public function __construct(private readonly UsersRepository $users)
    {
    }

    /**
     * @param  array<string, mixed>  $input
     * @return array{accessToken: string, tokenType: string}
     *
     * @throws AuthenticationException
     */
    public function login(array $input): array
    {
        $email = strtolower(trim((string) $input['email']));
        $user = $this->users->findByEmail($email);

        if ($user === null || ! Hash::check((string) $input['password'], (string) $user->password)) {
            throw new AuthenticationException('Invalid credentials');
        }

        $token = $user->createToken('api', ['*'])->plainTextToken;

        return [
            'accessToken' => $token,
            'tokenType' => 'Bearer',
        ];
    }
}

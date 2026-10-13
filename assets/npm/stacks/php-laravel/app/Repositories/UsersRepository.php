<?php

namespace App\Repositories;

use App\Models\User;

class UsersRepository
{
    public function findByEmail(string $email): ?User
    {
        return User::query()->where('email', $email)->first();
    }

    /**
     * @return array{0: list<User>, 1: int}
     */
    public function page(int $page, int $limit): array
    {
        $offset = ($page - 1) * $limit;
        $items = User::query()->orderByDesc('created_at')->skip($offset)->take($limit)->get()->all();
        $total = User::query()->count();
        return [$items, $total];
    }

    /**
     * @param  array{email: string, password: string, role: string}  $data
     */
    public function create(array $data): User
    {
        return User::query()->create($data);
    }
}

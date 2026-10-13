<?php

use App\Models\User;

it('rejects login with bad email', function () {
    $res = $this->postJson('/api/auth/login', [
        'email' => 'not-an-email',
        'password' => 'Str0ngPass',
    ]);
    $res->assertStatus(422);
});

it('rejects login with short password', function () {
    $res = $this->postJson('/api/auth/login', [
        'email' => 'user@example.test',
        'password' => 'short',
    ]);
    $res->assertStatus(422);
});

it('issues a token on valid credentials', function () {
    $user = User::query()->create([
        'email' => 'login@example.test',
        'password' => bcrypt('Str0ngPass'),
        'role' => 'USER',
    ]);

    $res = $this->postJson('/api/auth/login', [
        'email' => $user->email,
        'password' => 'Str0ngPass',
    ]);

    $res->assertOk();
    $res->assertJsonStructure(['accessToken', 'tokenType']);
});

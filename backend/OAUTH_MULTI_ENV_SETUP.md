# GitHub OAuth Multi-Environment Setup

This document explains how GitHub OAuth works across multiple environments (production, preview deployments, contributor branches) using a **single OAuth callback URL**.

## Problem

GitHub OAuth Apps only allow **ONE Authorization Callback URL**, but Vercel preview deployments create different URLs for each PR:
- Production: `https://grainlify.0xo.in`
- Preview: `https://grainlify-xyz.vercel.app`
- Fork Preview: `https://grainlify-git-fork-abc.vercel.app`

## Solution

Use **ONE stable OAuth callback** (production backend) for all environments, and let the backend redirect users back to the correct frontend after authentication.

## How It Works

### 1. Frontend Initiates OAuth

When a user clicks "Sign in with GitHub", the frontend:
- Captures its own origin: `window.location.origin`
- Passes it as a `redirect` query parameter to the backend

```typescript
// frontend/src/shared/api/client.ts
export const getGitHubLoginUrl = () => {
  const redirectAfterLogin = window.location.origin;
  return `${API_BASE_URL}/auth/github/login/start?redirect=${encodeURIComponent(redirectAfterLogin)}`;
};
```

**Examples:**
- Production: `https://api.grainlify.com/auth/github/login/start?redirect=https%3A%2F%2Fgrainlify.0xo.in`
- Preview: `https://api.grainlify.com/auth/github/login/start?redirect=https%3A%2F%2Fgrainlify-xyz.vercel.app`

### 2. Backend Stores Redirect URI

The backend `LoginStart` handler:
- Accepts the `redirect` query parameter
- Validates it's a valid URL
- Stores it in the `oauth_states` table along with the OAuth state
- Redirects user to GitHub OAuth

```go
// Backend stores redirect_uri in oauth_states table
INSERT INTO oauth_states (state, user_id, kind, expires_at, redirect_uri)
VALUES ($1, NULL, 'github_login', $2, $3)
```

### 3. GitHub OAuth Flow

- User authorizes on GitHub
- GitHub redirects to the **single callback URL** (production backend)
- Backend receives the OAuth code and state

**Important:** The GitHub OAuth App callback URL is always:
```
https://grainlify-production.up.railway.app/auth/github/login/callback
```

This URL is registered in GitHub OAuth App settings and **never changes**.

### 4. Backend Redirects to Correct Frontend

After successful authentication, the backend:
- Retrieves the stored `redirect_uri` from the `oauth_states` table
- Issues a JWT token
- Redirects user back to: `{redirect_uri}/auth/callback?token={jwt}&github={username}`

**Priority order:**
1. Stored `redirect_uri` from frontend (supports all environments)
2. `GITHUB_LOGIN_SUCCESS_REDIRECT_URL` config (fallback)
3. `FRONTEND_BASE_URL` config (fallback)

## Configuration

### GitHub OAuth App Settings

**Callback URL (set once, never change):**
```
https://grainlify-production.up.railway.app/auth/github/login/callback
```

### Backend Environment Variables

**Required:**
```bash
GITHUB_OAUTH_CLIENT_ID=your_client_id
GITHUB_OAUTH_CLIENT_SECRET=your_client_secret
GITHUB_OAUTH_REDIRECT_URL=https://grainlify-production.up.railway.app/auth/github/login/callback
```

**Optional (fallbacks):**
```bash
GITHUB_LOGIN_SUCCESS_REDIRECT_URL=https://grainlify.0xo.in/auth/callback  # Fallback if redirect param not provided
FRONTEND_BASE_URL=https://grainlify.0xo.in  # Another fallback
```

### Database Migration

Run the migration to add `redirect_uri` column:
```bash
# Migration is automatically applied if AUTO_MIGRATE=true
# Or run manually:
./cmd/migrate
```

Migration file: `migrations/000024_add_redirect_uri_to_oauth_states.up.sql`

## Benefits

✅ **Single OAuth callback URL** - Works with GitHub's limitation  
✅ **Works everywhere** - Production, preview, forks, localhost  
✅ **No configuration per environment** - Frontend automatically passes its origin  
✅ **Secure** - Backend validates redirect URLs  
✅ **Backward compatible** - Falls back to config if redirect param not provided  

## Flow Diagram

```
1. User on Preview → Clicks "Sign in with GitHub"
   ↓
2. Frontend: GET /auth/github/login/start?redirect=https://preview.vercel.app
   ↓
3. Backend: Stores redirect_uri, redirects to GitHub
   ↓
4. GitHub: User authorizes, redirects to production callback
   ↓
5. Backend: Processes OAuth, retrieves stored redirect_uri
   ↓
6. Backend: Redirects to https://preview.vercel.app/auth/callback?token=...
   ↓
7. User: Lands back on preview deployment with token
```

## Testing

### Test on Production
1. Visit production URL
2. Click "Sign in with GitHub"
3. Should redirect back to production after auth

### Test on Preview
1. Visit preview deployment URL
2. Click "Sign in with GitHub"
3. Should redirect back to preview URL after auth

### Test Locally
1. Run frontend locally (e.g., `http://localhost:5173`)
2. Click "Sign in with GitHub"
3. Should redirect back to `http://localhost:5173/auth/callback` after auth

## Troubleshooting

**Issue:** User not redirected to correct frontend
- Check that frontend is passing `redirect` parameter
- Check browser console for errors
- Verify `redirect_uri` is stored in `oauth_states` table

**Issue:** Invalid redirect_uri error
- Ensure frontend passes a valid URL (with protocol: `https://` or `http://`)
- Check backend logs for validation errors

**Issue:** Falls back to config URL
- This is expected if `redirect` parameter is not provided
- Ensure frontend code is updated to pass the parameter


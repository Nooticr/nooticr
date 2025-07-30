# React Application Context Guide

## Core Technologies
- **UI Framework**: `shadcn/ui` (React version)
- **State Management**: `Zustand` (lightweight alternative to Pinia)
- **GraphQL**: Apollo Client
- **Testing**: `Playwright` (real E2E tests without mocks)
- **Component Library**: Reusable components in `/components`

---

## Project Structure
```bash
src/
├── apollo/
│   └── client.ts              # Apollo Client configuration
├── graphql/
│   ├── mutations/             # All mutation operations
│   │   ├── login.ts
│   │   ├── signup.ts
│   │   └── ...
│   ├── queries/               # All query operations
│   │   └── get-societaire-dossier.ts
│   └── subscriptions/         # Subscription handlers
│
├── interfaces/                # TypeScript interfaces
│   ├── account.ts
│   ├── demande-comm.ts
│   └── ...
│
├── enums/                     # TypeScript enums
│   ├── account-type.ts
│   ├── mission-statut.ts
│   └── ...
│
├── stores/                    # Zustand stores
│   ├── auth.store.ts          # Authentication store
│   ├── societaire.store.ts    # Societaire operations
│   └── ...
│
├── pages/                     # Route-level components
│   ├── SocietaireApp.tsx
│   ├── AssureurDashboard.tsx
│   └── ...
│
└── components/                # Reusable components
    ├── ui/                    # shadcn/ui components
    ├── buttons/
    ├── cards/
    └── ...
```

---

## Implementation Rules

### 1. Component Architecture
- **Pages**: Only route-entry components in `/pages`. Must be composed of reusable components.
- **Components**: Break pages into smaller components in `/components`.

```tsx
// pages/SocietaireDashboard.tsx
import { DocumentUploadCard, MissionTimeline } from '@/components'
import { useSocietaireStore } from '@/stores'

const SocietaireDashboard = () => {
  const { documents } = useSocietaireStore()
  
  return (
    <DashboardLayout>
      <DocumentUploadCard onUpload={handleUpload} />
      <MissionTimeline items={documents} />
    </DashboardLayout>
  )
}
```

### 2. GraphQL Handling
- **Never call GraphQL directly in components**:
```tsx
// ❌ BAD (in component)
import { LOGIN_MUTATION } from '@/graphql/mutations/login'

// ✅ GOOD (via store)
import { useAuthStore } from '@/stores/auth'
const { login } = useAuthStore()
login({ email, password })
```

### 3. Zustand Stores Structure
Each store should encapsulate:
- State (reactive data)
- Actions (GraphQL operations + business logic)

```ts
// stores/auth.store.ts
import create from 'zustand'
import { LOGIN_MUTATION } from '@/graphql/mutations/login'
import { apolloClient } from '@/apollo/client'

interface AuthState {
  user: Account | null
  login: (credentials: LoginInput) => Promise<void>
}

export const useAuthStore = create<AuthState>((set) => ({
  user: null,
  login: async (credentials) => {
    const { data } = await apolloClient.mutate({
      mutation: LOGIN_MUTATION,
      variables: credentials
    })
    set({ user: data.login })
  }
}))
```

### 4. Testing Requirements
- **Playwright**:
  - Use `data-testid` attributes:
  ```tsx
  <Button data-testid="login-submit">Sign In</Button>
  ```
  - Real E2E tests:
  ```ts
  // tests/societaire-login.spec.ts
  import { test, expect } from '@playwright/test'

  test('Societaire login', async ({ page }) => {
    await page.goto('/societaire-login')
    await page.getByTestId('email-input').fill('test@societaire.fr')
    await page.getByTestId('login-submit').click()
    await expect(page).toHaveURL('/societaire-dashboard')
  })
  ```

### 5. Apollo Client Configuration
```ts
// apollo/client.ts
import { ApolloClient, InMemoryCache, createHttpLink } from '@apollo/client'

const httpLink = createHttpLink({
  uri: import.meta.env.VITE_GRAPHQL_ENDPOINT,
})

export const apolloClient = new ApolloClient({
  link: httpLink,
  cache: new InMemoryCache(),
})
```

---

## CI/CD Pipeline (GitHub Actions)

### Workflow Structure (`.github/workflows/main.yml`):
```yaml
name: React CI/CD
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  type-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 20
      - run: npm ci
      - run: npm run type-check

  e2e-tests:
    needs: type-check
    runs-on: ubuntu-latest
    timeout-minutes: 30
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 20
      - run: npm ci
      - run: npx playwright install
      - run: npm run build
      - run: npm run preview & npx wait-on http://localhost:4173
      - run: npx playwright test
        env:
          BASE_URL: http://localhost:4173
          TEST_USER: ${{ secrets.TEST_USER }}
          TEST_PASSWORD: ${{ secrets.TEST_PASSWORD }}

  release:
    needs: e2e-tests
    if: github.ref == 'refs/heads/main'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Bump version
        uses: phips28/gh-action-bump-version@v10
        with:
          tag-prefix: 'v'
          commit-message: 'chore(release): v{{version}}'
          default-bump: patch
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
      
      - name: Create Release
        uses: actions/create-release@v2
        with:
          tag_name: v${{ env.NEW_VERSION }}
          release_name: Release v${{ env.NEW_VERSION }}
          body: Production release
          draft: false
          prerelease: false
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}

  dockerize:
    needs: release
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build Docker image
        run: |
          docker build -t ghcr.io/${{ github.repository }}:${{ env.NEW_VERSION }} .
      - name: Push to GHCR
        uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}
      - run: docker push ghcr.io/${{ github.repository }}:${{ env.NEW_VERSION }}
```

### Dockerfile for React (Vite)
```dockerfile
# Builder stage
FROM node:20-alpine as builder
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build

# Production stage
FROM nginx:stable-alpine
COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
```

### nginx.conf
```nginx
server {
  listen 80;
  server_name localhost;

  location / {
    root /usr/share/nginx/html;
    index index.html;
    try_files $uri $uri/ /index.html;
  }

  error_page 500 502 503 504 /50x.html;
  location = /50x.html {
    root /usr/share/nginx/html;
  }
}
```

---

## React-Specific Best Practices

### 1. Component Design
- **Functional Components**: Use hooks for state and effects
- **Custom Hooks**: For reusable logic
```tsx
// hooks/useDocumentUpload.ts
const useDocumentUpload = () => {
  const [isUploading, setIsUploading] = useState(false)
  
  const uploadDocument = async (file: File) => {
    setIsUploading(true)
    try {
      await useDocumentStore.getState().upload(file)
    } finally {
      setIsUploading(false)
    }
  }
  
  return { isUploading, uploadDocument }
}
```

### 2. Error Boundaries
```tsx
// components/ErrorBoundary.tsx
import { Component, ErrorInfo, ReactNode } from 'react'

interface Props {
  children: ReactNode
  fallback: ReactNode
}

interface State {
  hasError: boolean
}

class ErrorBoundary extends Component<Props, State> {
  state: State = { hasError: false }

  static getDerivedStateFromError(_: Error): State {
    return { hasError: true }
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error("Uncaught error:", error, errorInfo)
  }

  render() {
    if (this.state.hasError) {
      return this.props.fallback
    }
    return this.props.children
  }
}

// Usage
<ErrorBoundary fallback={<ErrorScreen />}>
  <MissionTimeline />
</ErrorBoundary>
```

### 3. GraphQL Codegen
Add to workflow for type safety:
```yaml
# package.json
"scripts": {
  "codegen": "graphql-codegen --config codegen.yml"
}

# codegen.yml
overwrite: true
schema: "http://localhost:4000/graphql"
documents: "src/graphql/**/*.ts"
generates:
  src/generated/graphql.ts:
    plugins:
      - "typescript"
      - "typescript-operations"
      - "typescript-react-apollo"
```

---

## Issue Management Templates
Same as Vue implementation:
```
.github/
├── ISSUE_TEMPLATE/
│   ├── bug_report.md
│   ├── feature_request.md
│   └── improvement.md
```

## License (MIT)
Same MIT license file in project root

---

## React-Specific CI/CD Considerations

### 1. React Testing Library
Add component testing layer:
```yaml
# Additional CI job
component-tests:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: actions/setup-node@v4
    - run: npm ci
    - run: npm test -- --coverage
```

### 2. Vite-Specific Build
Optimize build process:
```dockerfile
# Dockerfile with multi-stage build for Vite
FROM node:20-alpine as builder
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build

FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
```

### 3. Playwright with React Server
Update E2E job to run preview server:
```yaml
e2e-tests:
  # ...
  steps:
    - run: npm run build
    - run: npm run preview & npx wait-on http://localhost:4173
    - run: npx playwright test
```

---

## Critical React-Specific Checks
1. **Hooks Rules**: Verify compliance with Rules of Hooks
2. **Memoization**: Use `useMemo`/`useCallback` appropriately
3. **Context Optimization**: Avoid unnecessary re-renders with context
4. **Bundle Analysis**: Add `@rollup/plugin-visualizer`
5. **Strict Mode**: Keep React Strict Mode enabled
6. **Concurrent Features**: Use `Suspense` for data fetching
7. **Server Components**: If using Next.js, leverage RSC architecture

This React implementation maintains the same core principles as the Vue.js version while leveraging React-specific patterns and best practices for optimal performance and maintainability.
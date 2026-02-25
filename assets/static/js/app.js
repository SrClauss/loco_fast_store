// API client
window.api = {
  baseURL: '/api',
  
  async request(endpoint, options = {}) {
    const token = localStorage.getItem('auth_token');
    const headers = {
      'Content-Type': 'application/json',
      ...options.headers,
    };
    
    if (token) {
      headers['Authorization'] = `Bearer ${token}`;
    }
    
    try {
      const response = await fetch(`${this.baseURL}${endpoint}`, {
        ...options,
        headers,
      });
      
      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.message || 'Request failed');
      }
      
      return await response.json();
    } catch (error) {
      console.error('API Error:', error);
      throw error;
    }
  },
  
  get(endpoint) {
    return this.request(endpoint);
  },
  
  post(endpoint, data) {
    return this.request(endpoint, {
      method: 'POST',
      body: JSON.stringify(data),
    });
  },
  
  put(endpoint, data) {
    return this.request(endpoint, {
      method: 'PUT',
      body: JSON.stringify(data),
    });
  },
  
  delete(endpoint) {
    return this.request(endpoint, {
      method: 'DELETE',
    });
  }
};

// Toast notifications
window.toast = {
  show(message, type = 'info') {
    Alpine.store('toasts').add({
      id: Date.now(),
      message,
      type,
    });
  },
  
  success(message) {
    this.show(message, 'success');
  },
  
  error(message) {
    this.show(message, 'error');
  },
  
  warning(message) {
    this.show(message, 'warning');
  },
  
  info(message) {
    this.show(message, 'info');
  }
};

// Format helpers
window.formatCurrency = (value) => {
  return new Intl.NumberFormat('pt-BR', {
    style: 'currency',
    currency: 'BRL'
  }).format(value / 100);
};

window.formatDate = (date) => {
  return new Intl.DateTimeFormat('pt-BR', {
    day: '2-digit',
    month: 'short',
    year: 'numeric',
  }).format(new Date(date));
};

window.formatDateTime = (date) => {
  return new Intl.DateTimeFormat('pt-BR', {
    day: '2-digit',
    month: 'short',
    year: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  }).format(new Date(date));
};

// Alpine.js stores
document.addEventListener('alpine:init', () => {
  // Auth store
  Alpine.store('auth', {
    user: null,
    token: localStorage.getItem('auth_token'),
    
    async login(email, password) {
      try {
        const response = await api.post('/auth/login', { email, password });
        this.token = response.token;
        this.user = response.user;
        localStorage.setItem('auth_token', response.token);
        return response;
      } catch (error) {
        throw error;
      }
    },
    
    logout() {
      this.token = null;
      this.user = null;
      localStorage.removeItem('auth_token');
      window.location.href = '/admin/login';
    },
    
    async fetchUser() {
      if (!this.token) return;
      try {
        this.user = await api.get('/auth/me');
      } catch (error) {
        this.logout();
      }
    }
  });
  
  // Toast store
  Alpine.store('toasts', {
    items: [],
    
    add(toast) {
      this.items.push(toast);
      setTimeout(() => this.remove(toast.id), 5000);
    },
    
    remove(id) {
      this.items = this.items.filter(t => t.id !== id);
    }
  });
  
  // Sidebar store
  Alpine.store('sidebar', {
    open: window.innerWidth >= 1024,
    
    toggle() {
      this.open = !this.open;
    }
  });
  
  // Modal store
  Alpine.store('modal', {
    isOpen: false,
    title: '',
    content: '',
    
    open(title, content) {
      this.title = title;
      this.content = content;
      this.isOpen = true;
    },
    
    close() {
      this.isOpen = false;
    }
  });
});

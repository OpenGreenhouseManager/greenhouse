import { provideRouter } from '@angular/router';

import { provideHttpClient, withInterceptors } from '@angular/common/http';
import type { ApplicationConfig } from '@angular/core';
import { definePreset } from '@primeuix/themes';
import Lara from '@primeuix/themes/lara';
import { providePrimeNG } from 'primeng/config';
import { authInterceptor } from './_interceptors/auth.interceptor';
import { routes } from './app.routes';

// Sage + sakura light theme. Only overrides are listed here; everything else
// (primitive palettes, component tokens) comes from Lara.
const MyTheme = definePreset(Lara, {
  primitive: {
    borderRadius: {
      none: '0',
      xs: '3px',
      sm: '6px',
      md: '8px',
      lg: '12px',
      xl: '16px',
    },
  },
  semantic: {
    transitionDuration: '0.2s',
    focusRing: {
      width: '2px',
      style: 'solid',
      color: '{primary.300}',
      offset: '2px',
      shadow: 'none',
    },
    primary: {
      50: '#f2f6f0',
      100: '#e0eadc',
      200: '#c3d6bc',
      300: '#9fbd96',
      400: '#78a06e',
      500: '#4f7a4a',
      600: '#456c41',
      700: '#3f6640',
      800: '#33532f',
      900: '#263f24',
      950: '#17251a',
    },
    formField: {
      paddingX: '0.75rem',
      paddingY: '0.55rem',
      borderRadius: '{border.radius.md}',
      focusRing: {
        width: '3px',
        style: 'solid',
        color: '#fbe9ee',
        offset: '0',
        shadow: 'none',
      },
    },
    content: {
      borderRadius: '{border.radius.lg}',
    },
    overlay: {
      select: {
        borderRadius: '{border.radius.md}',
        shadow: '0 8px 24px rgba(36, 48, 31, 0.12)',
      },
      popover: {
        borderRadius: '{border.radius.md}',
        padding: '0.75rem',
        shadow: '0 8px 24px rgba(36, 48, 31, 0.12)',
      },
      modal: {
        borderRadius: '{border.radius.xl}',
        padding: '1.5rem',
        shadow: '0 20px 48px rgba(36, 48, 31, 0.18)',
      },
    },
    colorScheme: {
      light: {
        surface: {
          0: '#ffffff',
          50: '#f7f6f1',
          100: '#eef2ea',
          200: '#e3e6dc',
          300: '#ccd2c3',
          400: '#a3ab98',
          500: '#7c8573',
          600: '#66705f',
          700: '#47523f',
          800: '#34402e',
          900: '#24301f',
          950: '#151c12',
        },
        primary: {
          color: '{primary.500}',
          contrastColor: '#ffffff',
          hoverColor: '{primary.700}',
          activeColor: '{primary.800}',
        },
        highlight: {
          background: '#fbe9ee',
          focusBackground: '#f6d6de',
          color: '{primary.800}',
          focusColor: '{primary.900}',
        },
        mask: {
          background: 'rgba(36, 48, 31, 0.35)',
          color: '{surface.200}',
        },
        formField: {
          background: '{surface.0}',
          borderColor: '{surface.300}',
          hoverBorderColor: '{primary.300}',
          focusBorderColor: '{primary.500}',
          color: '{surface.900}',
          placeholderColor: '{surface.500}',
          floatLabelColor: '{surface.600}',
          floatLabelFocusColor: '{primary.600}',
          iconColor: '{surface.500}',
        },
        text: {
          color: '{surface.900}',
          hoverColor: '{surface.950}',
          mutedColor: '{surface.600}',
          hoverMutedColor: '{surface.700}',
        },
        content: {
          background: '{surface.0}',
          hoverBackground: '{surface.100}',
          borderColor: '{surface.200}',
          color: '{text.color}',
          hoverColor: '{text.hover.color}',
        },
      },
    },
  },
  components: {
    // Soft pill tabs on the page background (radius/shadow in styles.scss)
    tabs: {
      tablist: {
        background: 'transparent',
        borderWidth: '0',
      },
      tab: {
        background: 'transparent',
        hoverBackground: '{surface.100}',
        activeBackground: '{surface.0}',
        borderWidth: '1px',
        borderColor: 'transparent',
        hoverBorderColor: 'transparent',
        activeBorderColor: '{surface.200}',
        margin: '0 0.25rem 0 0',
        padding: '0.45rem 1rem',
        fontWeight: '600',
      },
      tabpanel: {
        background: 'transparent',
        padding: '1rem 0 0 0',
      },
      navButton: {
        background: '{surface.50}',
        shadow: '0px 0px 10px 50px rgba(247, 246, 241, 0.6)',
      },
    },
  },
});

export const appConfig: ApplicationConfig = {
  providers: [
    provideRouter(routes),
    provideHttpClient(withInterceptors([authInterceptor])),
    providePrimeNG({
      // PrimeUI Community License key (not a secret). Without it PrimeNG 22+
      // shows an invalid-license banner.
      license:
        'eyJpZCI6IjhhNzQ4MDkyLWE3OGYtNDRmZS04OTU5LWIyZjFiOTZiZDQyMyIsInByb2R1Y3QiOiJwcmltZXVpIiwidGllciI6ImNvbW11bml0eSIsInR5cGUiOiJkZXYiLCJpYXQiOjE3OTI2NzQxMDQsImV4cCI6MTgyNDIxMDEwNH0.CJeI4Pgi6QdA2bheRODHjIwGq49JxN06sOF1hkr8pksOMVpSNgoVOOHvHaRs78R4yzRQYoRJOxiHny4EOGATBw',
      theme: {
        preset: MyTheme,
        options: {
          darkModeSelector: '',
        },
      },
    }),
  ],
};

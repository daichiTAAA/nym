import React from 'react';
import { Box, Button, Modal, Stack, SxProps, Typography } from '@mui/material';
import ArrowBackIosNewIcon from '@mui/icons-material/ArrowBackIosNew';
import CloseIcon from '@mui/icons-material/Close';
import ErrorOutline from '@mui/icons-material/ErrorOutline';
import { modalStyle } from './styles';

export const SimpleModal: React.FC<{
  open: boolean;
  hideCloseIcon?: boolean;
  displayErrorIcon?: boolean;
  headerStyles?: SxProps;
  subHeaderStyles?: SxProps;
  onClose?: () => void;
  onOk?: () => Promise<void>;
  onSecondaryAction?: () => void;
  header: string;
  subHeader?: string;
  okLabel: string;
  okDisabled?: boolean;
  sx?: SxProps;
}> = ({
  open,
  hideCloseIcon,
  displayErrorIcon,
  headerStyles,
  subHeaderStyles,
  onClose,
  okDisabled,
  onOk,
  onSecondaryAction,
  header,
  subHeader,
  okLabel,
  sx,
  children,
}) => (
  <Modal open={open} onClose={onClose}>
    <Box sx={{ ...modalStyle, ...sx }}>
      {displayErrorIcon && <ErrorOutline color="error" sx={{ mb: 3 }} />}
      <Stack direction="row" justifyContent="space-between" alignItems="center">
        <Typography fontSize={22} fontWeight={600} sx={{ ...headerStyles }}>
          {header}
        </Typography>
        {!hideCloseIcon && <CloseIcon onClick={onClose} cursor="pointer" />}
      </Stack>
      {subHeader && (
        <Typography
          mt={0.5}
          mb={3}
          fontSize="small"
          color={(theme) => theme.palette.text.secondary}
          sx={{ ...subHeaderStyles }}
        >
          {subHeader}
        </Typography>
      )}

      {children}

      <Box sx={{ display: 'flex', alignItems: 'center', gap: 2, mt: 2 }}>
        {onSecondaryAction && (
          <Button
            disableFocusRipple
            size="large"
            variant="outlined"
            startIcon={<ArrowBackIosNewIcon sx={{ width: 20 }} />}
            onClick={onSecondaryAction}
          />
        )}
        <Button variant="contained" fullWidth size="large" onClick={onOk} disabled={okDisabled}>
          {okLabel}
        </Button>
      </Box>
    </Box>
  </Modal>
);

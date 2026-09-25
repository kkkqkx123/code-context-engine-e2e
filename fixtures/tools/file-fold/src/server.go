package service

import (
	"errors"
	"fmt"
	"time"
)

type Config struct {
	Name     string
	Interval time.Duration
}

type Server struct {
	cfg     Config
	started bool
}

func NewServer(cfg Config) (*Server, error) {
	if cfg.Name == "" {
		return nil, errors.New("name is required")
	}
	return &Server{cfg: cfg}, nil
}

func (s *Server) Start() error {
	if s.started {
		return fmt.Errorf("server %s already started", s.cfg.Name)
	}
	s.started = true
	return nil
}

func (s *Server) Stop() error {
	s.started = false
	return nil
}

func (s *Server) Status() string {
	if s.started {
		return "running"
	}
	return "stopped"
}
